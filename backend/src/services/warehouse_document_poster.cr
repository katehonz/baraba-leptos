class WarehouseDocumentPoster
  class Error < Exception
  end

  def self.post!(doc : WarehouseDocument, company : Company, allow_negative : Bool) : WarehouseDocument
    raise Error.new("Вече е осчетоводен") if doc.status == "posted"
    PeriodService.validate_period_open!(company.id, doc.issue_date)

    type = WarehouseDocumentTypeQuery.new.code(doc.document_type).first
    raise Error.new("Неизвестен складов тип") unless type

    seq = doc.sequence_id || DocumentNumberService.sequence_for(company.id, "warehouse", doc.document_type).try(&.id)
    raise Error.new("Няма номерационна верига") unless seq

    number = doc.document_number || DocumentNumberService.allocate!(seq)
    movement = type.saft_movement_type

    lines = WarehouseDocumentLineQuery.new.warehouse_document_id(doc.id)
    raise Error.new("Няма редове") if lines.select_count == 0

    AppDatabase.transaction do
      lines.each do |line|
        apply_line(doc, company, type, line, number, movement, allow_negative)
      end
      SaveWarehouseDocument.update!(doc,
        status: "posted",
        document_number: number,
        sequence_id: seq,
        movement_type: movement
      )
    end
    WarehouseDocumentQuery.new.id(doc.id).first.not_nil!
  end

  def self.unpost!(doc : WarehouseDocument, company : Company, allow_negative : Bool) : WarehouseDocument
    raise Error.new("Не е осчетоводен") unless doc.status == "posted"
    PeriodService.validate_period_open!(company.id, doc.issue_date)

    AppDatabase.transaction do
      StockTransactionQuery.new.warehouse_document_id(doc.id).each do |st|
        reverse = !st.is_incoming
        StockBalanceService.apply!(
          company,
          st.product_id,
          st.warehouse_id,
          st.quantity,
          st.unit_price,
          reverse,
          true
        )
        DeleteStockTransaction.delete!(st)
      end
      if je_id = doc.journal_entry_id
        if je = JournalEntryQuery.new.id(je_id).first?
          SaveJournalEntry.update!(je, status: "reversed", vat_purchase_operation: nil, vat_sales_operation: nil)
        end
      end
      SaveWarehouseDocument.update!(doc, status: "cancelled", journal_entry_id: nil)
    end
    WarehouseDocumentQuery.new.id(doc.id).first.not_nil!
  end

  def self.ensure_for_invoice!(invoice : Invoice, company : Company, kind : String) : WarehouseDocument
    if existing = WarehouseDocumentQuery.new.invoice_id(invoice.id).first?
      return existing
    end
    loc_id = invoice.location_id
    raise Error.new("Липсва търговски обект със склад") unless loc_id
    loc = CompanyLocationQuery.new.id(loc_id).first
    raise Error.new("Обектът не държи запас") unless loc && loc.holds_stock

    type = WarehouseDocumentTypeQuery.new.code(kind).first
    raise Error.new("Неизвестен складов тип") unless type

    from_id = type.needs_from ? loc_id : nil
    to_id = type.needs_to ? loc_id : nil

    orig_id = nil.as(Int64?)
    if kind == "PURCHASE_RETURN"
      if orig = invoice.original_invoice_id
        orig_id = WarehouseDocumentQuery.new.invoice_id(orig).first?.try(&.id)
      end
    end

    doc = SaveWarehouseDocument.create!(
      company_id: company.id,
      document_type: kind,
      issue_date: invoice.issue_date,
      status: "draft",
      from_location_id: from_id,
      to_location_id: to_id,
      invoice_id: invoice.id,
      original_warehouse_document_id: orig_id,
      movement_type: type.saft_movement_type,
      notes: "Автоматично от документ #{invoice.invoice_number}"
    )

    n = 0
    InvoiceLineQuery.new.invoice_id(invoice.id).each do |line|
      next unless inventory_line?(line)
      n += 1
      qty = line.quantity.abs
      price = line.unit_price.abs
      SaveWarehouseDocumentLine.create!(
        warehouse_document_id: doc.id,
        product_id: line.product_id.not_nil!,
        quantity: qty,
        unit_code: line.unit_code,
        unit_price: price,
        line_total: Money.round(qty * price),
        line_number: n
      )
    end
    raise Error.new("Няма складови редове") if n == 0
    WarehouseDocumentQuery.new.id(doc.id).first.not_nil!
  end

  def self.inventory_line?(line : InvoiceLine) : Bool
    return false unless line.product_id
    product = ProductQuery.new.id(line.product_id.not_nil!).first?
    return false unless product
    return false if product.goods_services_id == "02"
    return false unless product.product_type.try { |t| ["10", "20", "30", "40"].includes?(t) }
    !product.inventory_account_id.nil?
  end

  private def self.apply_line(doc, company, type, line, number, movement, allow_negative)
    if doc.document_type == "TRANSFER"
      from_wh = warehouse_for!(doc.from_location_id)
      to_wh = warehouse_for!(doc.to_location_id)
      avg = StockBalanceService.current_avg(company.id, line.product_id, from_wh.id)
      write_st(doc, company, line, from_wh, number, movement, false, avg)
      StockBalanceService.apply!(company, line.product_id, from_wh.id, line.quantity, avg, false, allow_negative)
      write_st(doc, company, line, to_wh, number, movement, true, avg)
      StockBalanceService.apply!(company, line.product_id, to_wh.id, line.quantity, avg, true, allow_negative)
    else
      incoming = type.default_is_incoming == true
      loc_id = incoming ? doc.to_location_id : doc.from_location_id
      wh = warehouse_for!(loc_id)
      price = incoming ? line.unit_price : StockBalanceService.current_avg(company.id, line.product_id, wh.id)
      write_st(doc, company, line, wh, number, movement, incoming, price)
      StockBalanceService.apply!(company, line.product_id, wh.id, line.quantity, price, incoming, allow_negative)
    end
  end

  private def self.warehouse_for!(location_id : Int64?) : Warehouse
    raise Error.new("Липсва локация") unless location_id
    WarehouseQuery.new.company_location_id(location_id).first || raise Error.new("Няма склад за обекта")
  end

  private def self.write_st(doc, company, line, warehouse, number, movement, incoming, price)
    SaveStockTransaction.create!(
      company_id: company.id,
      product_id: line.product_id,
      warehouse_id: warehouse.id,
      warehouse_document_id: doc.id,
      transaction_date: doc.issue_date,
      quantity: line.quantity,
      unit_price: price,
      total_amount: Money.round(line.quantity * price),
      is_incoming: incoming,
      movement_type: movement,
      document_number: number,
      document_date: doc.issue_date,
      document_type: doc.document_type,
      unit_of_measure: line.unit_code,
      book_value: Money.round(line.quantity * price),
      line_number: line.line_number
    )
  end
end
