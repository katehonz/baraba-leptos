require "../../../spec_helper"

private def auth_user : User
  UserFactory.create &.is_super_admin(true)
end

describe Api::WarehouseScans::Apply do
  it "creates a purchase draft and linked RECEIPT, and rejects both flags" do
    seed_uom
    seed_warehouse_doc_types
    user = auth_user
    company = SaveCompany.create!(name: "Приложи скан ООД", eik: "209111112")
    counterpart = SaveCounterpart.create!(company_id: company.id, name: "Вендор ООД", counterpart_type: "supplier")
    loc = SaveCompanyLocation.create!(
      company_id: company.id,
      name: "Основен склад",
      location_type: "WAREHOUSE",
      holds_stock: true,
      country: "BG"
    )
    product = SaveProduct.create!(
      company_id: company.id,
      code: "GOOD-1",
      name: "Стоманена тръба",
      product_type: "30",
      uom_standard: "C62",
      measure_unit: "C62"
    )

    scan = SaveWarehouseScan.create!(
      company_id: company.id,
      direction: "inbound",
      status: "reviewed",
      counterpart_id: counterpart.id,
      vendor_name: "Вендор ООД",
      invoice_number: "INV-88",
      document_date: Time.utc(2026, 9, 1),
      subtotal: 10.0,
      vat_amount: 2.0,
      total_amount: 12.0
    )
    SaveWarehouseScanLine.create!(
      warehouse_scan_id: scan.id,
      raw_name: "Тръба",
      raw_sku: "T-1",
      quantity: 2.0,
      unit_code: "C62",
      unit_price: 5.0,
      vat_rate: 20.0,
      mapping_status: "manual",
      line_number: 1,
      product_id: product.id
    )

    both = ApiClient.auth(user).exec_raw(
      Api::WarehouseScans::Apply.with(company.id.to_s, scan.id.to_s),
      {create_invoice: true, create_orphan_receipt: true, location_id: loc.id}.to_json
    )
    both.status_code.should eq(422)

    ok = ApiClient.auth(user).exec_raw(
      Api::WarehouseScans::Apply.with(company.id.to_s, scan.id.to_s),
      {create_invoice: true, location_id: loc.id}.to_json
    )
    ok.status_code.should eq(200), ok.body
    data = JSON.parse(ok.body)["data"]
    data["status"].as_s.should eq("applied")
    invoice_id = data["invoice_id"].as_i64
    wd_id = data["warehouse_document_id"].as_i64

    invoice = InvoiceQuery.new.id(invoice_id).first
    invoice.direction.should eq("purchase")
    invoice.status.should eq("draft")
    invoice.invoice_number.should eq("INV-88")
    InvoiceLineQuery.new.invoice_id(invoice.id).select_count.should eq(1)

    wd = WarehouseDocumentQuery.new.id(wd_id).first
    wd.document_type.should eq("RECEIPT")
    wd.invoice_id.should eq(invoice.id)
    wd.status.should eq("draft")
  end

  it "creates an orphan RECEIPT without an invoice" do
    seed_uom
    seed_warehouse_doc_types
    user = auth_user
    company = SaveCompany.create!(name: "Орфан скан ООД", eik: "209111113")
    loc = SaveCompanyLocation.create!(
      company_id: company.id,
      name: "Склад 2",
      location_type: "WAREHOUSE",
      holds_stock: true,
      country: "BG"
    )
    product = SaveProduct.create!(
      company_id: company.id,
      code: "GOOD-2",
      name: "Лист",
      product_type: "30",
      uom_standard: "C62",
      measure_unit: "C62"
    )
    scan = SaveWarehouseScan.create!(
      company_id: company.id,
      direction: "inbound",
      status: "reviewed",
      document_date: Time.utc(2026, 9, 2)
    )
    SaveWarehouseScanLine.create!(
      warehouse_scan_id: scan.id,
      raw_name: "Лист",
      quantity: 1.0,
      unit_code: "C62",
      unit_price: 3.0,
      mapping_status: "manual",
      line_number: 1,
      product_id: product.id
    )

    response = ApiClient.auth(user).exec_raw(
      Api::WarehouseScans::Apply.with(company.id.to_s, scan.id.to_s),
      {create_invoice: false, create_orphan_receipt: true, location_id: loc.id}.to_json
    )
    response.status_code.should eq(200), response.body
    data = JSON.parse(response.body)["data"]
    data["invoice_id"].raw.should be_nil
    wd = WarehouseDocumentQuery.new.id(data["warehouse_document_id"].as_i64).first
    wd.document_type.should eq("RECEIPT")
    wd.invoice_id.should be_nil
  end
end

describe Api::WarehouseScans::Create do
  it "rejects upload when Mistral is disabled" do
    user = auth_user
    company = SaveCompany.create!(name: "Без мистрал ООД", eik: "209111114")
    response = ApiClient.auth(user).exec_raw(
      Api::WarehouseScans::Create.with(company.id.to_s),
      {file_content: "nope"}.to_json
    )
    response.status_code.should eq(400)
  end
end

describe Api::WarehouseScans::Update do
  it "remaps unmapped lines when a counterpart is chosen" do
    seed_uom
    user = auth_user
    company = SaveCompany.create!(name: "Контрагент мап ООД", eik: "209111117")
    counterpart = SaveCounterpart.create!(company_id: company.id, name: "Дост 2", counterpart_type: "supplier")
    product = SaveProduct.create!(company_id: company.id, code: "M-2", name: "Шайба", product_type: "30", uom_standard: "C62", measure_unit: "C62")
    VendorProductMappingService.remember!(
      company.id, product.id, counterpart.id, "Дост 2", "W-9", "Шайба 8", nil, "C62", true
    )
    scan = SaveWarehouseScan.create!(
      company_id: company.id,
      direction: "inbound",
      status: "pending"
    )
    line = SaveWarehouseScanLine.create!(
      warehouse_scan_id: scan.id,
      raw_name: "Шайба 8",
      raw_sku: "W-9",
      quantity: 1.0,
      mapping_status: "unmapped",
      line_number: 1
    )

    response = ApiClient.auth(user).exec_raw(
      Api::WarehouseScans::Update.with(company.id.to_s, scan.id.to_s),
      {counterpart_id: counterpart.id}.to_json
    )
    response.status_code.should eq(200)
    JSON.parse(response.body)["data"]["status"].as_s.should eq("reviewed")
    reloaded = WarehouseScanLineQuery.new.id(line.id).first
    reloaded.product_id.should eq(product.id)
    reloaded.mapping_status.should eq("auto")
  end
end

describe Api::WarehouseScans::UpdateLine do
  it "maps a line to a product and marks the scan reviewed" do
    seed_uom
    user = auth_user
    company = SaveCompany.create!(name: "Ред скан ООД", eik: "209111115")
    counterpart = SaveCounterpart.create!(company_id: company.id, name: "Дост", counterpart_type: "supplier")
    product = SaveProduct.create!(company_id: company.id, code: "M-1", name: "Материал", product_type: "30", uom_standard: "C62", measure_unit: "C62")
    scan = SaveWarehouseScan.create!(
      company_id: company.id,
      direction: "inbound",
      status: "pending",
      counterpart_id: counterpart.id
    )
    line = SaveWarehouseScanLine.create!(
      warehouse_scan_id: scan.id,
      raw_name: "Материал Х",
      raw_sku: "MX",
      quantity: 1.0,
      mapping_status: "unmapped",
      line_number: 1
    )

    response = ApiClient.auth(user).exec_raw(
      Api::WarehouseScans::UpdateLine.with(company.id.to_s, scan.id.to_s, line.id.to_s),
      {product_id: product.id}.to_json
    )
    response.status_code.should eq(200)
    JSON.parse(response.body)["data"]["status"].as_s.should eq("reviewed")
    reloaded = WarehouseScanLineQuery.new.id(line.id).first
    reloaded.product_id.should eq(product.id)
    reloaded.mapping_status.should eq("manual")
    VendorProductMappingQuery.new.company_id(company.id).vendor_sku("MX").first?.should_not be_nil
  end
end
