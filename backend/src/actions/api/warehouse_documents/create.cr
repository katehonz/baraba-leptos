class Api::WarehouseDocuments::Create < ApiAction
  include Api::InvoiceParams

  post "/api/companies/:company_id/warehouse_documents" do
    cid = company_id.to_i64
    authorize!(Permission::WAREHOUSE_CREATE, cid)
    data = nested_or_flat(params.from_json, "warehouse_document")
    doc_type = json_str(data["document_type"]?) || "RECEIPT"
    if WarehouseDocumentTypeQuery.new.code(doc_type).system_only(true).first?
      response.status_code = 422
      return json({success: false, error: "Системен тип — създава се от фактура"})
    end
    issue_date = parse_optional_date(data["issue_date"]?) || Time.utc
    doc = SaveWarehouseDocument.create!(
      company_id: cid,
      document_type: doc_type,
      issue_date: issue_date,
      status: "draft",
      from_location_id: json_i64(data["from_location_id"]?),
      to_location_id: json_i64(data["to_location_id"]?),
      notes: json_str(data["notes"]?)
    )
    if lines = data["lines"]?.try(&.as_a?)
      lines.each_with_index do |ln, i|
        qty = json_f64(ln["quantity"]?) || 0.0
        price = json_f64(ln["unit_price"]?) || 0.0
        pid = json_i64(ln["product_id"]?)
        next unless pid
        SaveWarehouseDocumentLine.create!(
          warehouse_document_id: doc.id,
          product_id: pid,
          quantity: qty,
          unit_code: json_str(ln["unit_code"]?) || "C62",
          unit_price: price,
          line_total: Money.round(qty * price),
          line_number: i + 1
        )
      end
    end
    json({success: true, data: {id: doc.id, status: doc.status, document_type: doc.document_type}})
  end
end
