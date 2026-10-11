class WarehouseScanApplier
  class Error < Exception
  end

  def self.apply!(
    scan : WarehouseScan,
    company : Company,
    create_invoice : Bool,
    create_orphan_receipt : Bool,
    location_id : Int64?
  ) : NamedTuple(invoice: Invoice?, warehouse_document: WarehouseDocument?)
    raise Error.new("Вече е приложен") if scan.status == "applied"
    raise Error.new("Отхвърлен скан") if scan.status == "rejected"
    if create_invoice && create_orphan_receipt
      raise Error.new("Не може едновременно фактура и самостоятелна складова разписка")
    end
    unless create_invoice || create_orphan_receipt
      raise Error.new("Изберете create_invoice или create_orphan_receipt")
    end

    lines = WarehouseScanLineQuery.new.warehouse_scan_id(scan.id).line_number.asc_order
    raise Error.new("Няма редове") if lines.select_count == 0

    invoice = nil.as(Invoice?)
    wd = nil.as(WarehouseDocument?)

    AppDatabase.transaction do
      if create_invoice
        invoice = build_invoice!(scan, company, location_id, lines)
        if location_id && product_lines?(lines)
          if WarehouseDocumentQuery.new.invoice_id(invoice.not_nil!.id).first?
            raise Error.new("Вече има складов документ към тази фактура")
          end
          wd = build_stock_doc!(scan, company, location_id, lines, invoice.not_nil!.id)
        end
      else
        raise Error.new("Липсва търговски обект") unless location_id
        if lines.any? { |l| l.product_id.nil? }
          raise Error.new("Има несъпоставени редове — за складова разписка всеки ред трябва да има артикул")
        end
        wd = build_stock_doc!(scan, company, location_id, lines, nil)
      end

      SaveWarehouseScan.update!(scan,
        status: "applied",
        invoice_id: invoice.try(&.id),
        warehouse_document_id: wd.try(&.id)
      )
    end

    {invoice: invoice, warehouse_document: wd}
  end

  private def self.product_lines?(lines) : Bool
    lines.any? { |l| l.product_id }
  end

  private def self.build_invoice!(scan, company, location_id, lines) : Invoice
    cp = scan.counterpart_id
    raise Error.new("Изберете контрагент преди прилагане") unless cp

    direction = scan.direction == "outbound" ? "sale" : "purchase"
    issue_date = scan.document_date || Time.utc
    has_inv = product_lines?(lines)
    number = direction == "purchase" ? scan.invoice_number : nil

    invoice = SaveInvoice.create!(
      company_id: company.id,
      counterpart_id: cp,
      invoice_number: number,
      document_type: "INVOICE",
      direction: direction,
      issue_date: issue_date,
      tax_event_date: issue_date,
      subtotal: scan.subtotal || 0.0,
      vat_amount: scan.vat_amount || 0.0,
      total_amount: scan.total_amount || 0.0,
      currency: company.currency,
      status: "draft",
      location_id: location_id,
      has_inventory: has_inv,
      notes: scan.vendor_name
    )

    n = 0
    lines.each do |line|
      desc = line.raw_name.presence || line.raw_sku.presence || "Ред #{line.line_number}"
      n += 1
      SaveInvoiceLine.create!(
        invoice_id: invoice.id,
        description: desc,
        quantity: line.quantity,
        unit_price: line.unit_price,
        vat_rate: line.vat_rate,
        product_id: line.product_id,
        unit_code: line.unit_code,
        line_number: n
      )
    end
    rebuilt = InvoiceLineQuery.new.invoice_id(invoice.id)
    subtotal = rebuilt.sum(&.line_total)
    vat = rebuilt.sum(&.vat_amount)
    SaveInvoice.update!(invoice, subtotal: subtotal, vat_amount: vat, total_amount: subtotal + vat)
    InvoicePoster.rebuild_json_cache(invoice, rebuilt)
    InvoiceQuery.new.id(invoice.id).first.not_nil!
  end

  private def self.build_stock_doc!(scan, company, location_id, lines, invoice_id : Int64?) : WarehouseDocument
    loc = CompanyLocationQuery.new.id(location_id).company_id(company.id).first?
    raise Error.new("Обектът не държи запас") unless loc && loc.holds_stock

    inbound = scan.direction != "outbound"
    doc_type = inbound ? "RECEIPT" : "ISSUE"
    type = WarehouseDocumentTypeQuery.new.code(doc_type).first?
    raise Error.new("Неизвестен складов тип") unless type

    doc = SaveWarehouseDocument.create!(
      company_id: company.id,
      document_type: doc_type,
      issue_date: scan.document_date || Time.utc,
      status: "draft",
      from_location_id: type.needs_from ? location_id : nil,
      to_location_id: type.needs_to ? location_id : nil,
      invoice_id: invoice_id,
      notes: "От складово сканиране ##{scan.id}",
      movement_type: type.saft_movement_type
    )

    n = 0
    lines.each do |line|
      next unless line.product_id
      n += 1
      qty = line.quantity
      price = line.unit_price
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
end
