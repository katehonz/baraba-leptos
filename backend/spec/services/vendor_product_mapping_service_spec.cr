require "../spec_helper"

describe VendorProductMappingService do
  it "normalizes names and looks up by sku then normalized name" do
    seed_uom
    company = SaveCompany.create!(name: "Склад скан ООД", eik: "209111111")
    counterpart = SaveCounterpart.create!(company_id: company.id, name: "Доставчик АД", counterpart_type: "supplier")
    product = SaveProduct.create!(company_id: company.id, code: "P-1", name: "Болт", product_type: "30", uom_standard: "C62", measure_unit: "C62")

    mapping = VendorProductMappingService.remember!(
      company.id,
      product.id,
      counterpart.id,
      "Доставчик АД",
      "SKU-9",
      "Болт М8",
      nil,
      "C62",
      true
    )
    mapping.vendor_name_normalized.should eq("болт м8")
    mapping.confirmed.should be_true

    found = VendorProductMappingService.find(company.id, counterpart.id, "SKU-9", nil, nil)
    found.not_nil!.product_id.should eq(product.id)

    found2 = VendorProductMappingService.find(company.id, counterpart.id, nil, nil, "Болт, М8!")
    found2.not_nil!.id.should eq(mapping.id)

    VendorProductMappingService.auto_product_id(mapping, 0.5).should eq(product.id)
    unconfirmed = SaveVendorProductMapping.update!(mapping, confirmed: false, hit_count: 1)
    VendorProductMappingService.auto_product_id(unconfirmed, 0.9).should be_nil
  end

  it "auto-maps unmapped scan lines from a confirmed vendor map" do
    seed_uom
    company = SaveCompany.create!(name: "Автомап ООД", eik: "209111116")
    counterpart = SaveCounterpart.create!(company_id: company.id, name: "Вендор Б", counterpart_type: "supplier")
    product = SaveProduct.create!(company_id: company.id, code: "P-2", name: "Гайка", product_type: "30", uom_standard: "C62", measure_unit: "C62")
    VendorProductMappingService.remember!(
      company.id, product.id, counterpart.id, "Вендор Б", "SKU-77", "Гайка М8", nil, "C62", true
    )
    scan = SaveWarehouseScan.create!(
      company_id: company.id,
      direction: "inbound",
      status: "pending",
      counterpart_id: counterpart.id
    )
    line = SaveWarehouseScanLine.create!(
      warehouse_scan_id: scan.id,
      raw_name: "Гайка М8",
      raw_sku: "SKU-77",
      quantity: 4.0,
      mapping_status: "unmapped",
      line_number: 1,
      confidence: 0.4
    )

    n = VendorProductMappingService.apply_to_unmapped_scan_lines!(company.id, scan)
    n.should eq(1)
    reloaded = WarehouseScanLineQuery.new.id(line.id).first
    reloaded.product_id.should eq(product.id)
    reloaded.mapping_status.should eq("auto")
  end
end

describe MistralWarehouseService do
  it "parses line-item JSON from the warehouse prompt" do
    raw = <<-JSON
    {"vendorName":"Метал ООД","vendorVat":"BG123","invoiceNumber":"0000000044","invoiceDate":"2026-09-01","subtotal":10,"vatAmount":2,"total":12,"confidence":0.9,"items":[{"name":"Тръба","sku":"T-1","quantity":2,"unit":"кг","unitPrice":5,"vatRate":20}]}
    JSON
    result = MistralWarehouseService.parse_extraction(raw)
    result.vendor_name.should eq("Метал ООД")
    result.invoice_number.should eq("0000000044")
    result.items.size.should eq(1)
    result.items.first.sku.should eq("T-1")
    result.items.first.quantity.should eq(2.0)
    MistralWarehouseService.guess_unit("кг").should eq("KGM")
    MistralWarehouseService.guess_unit("бр.").should eq("C62")
  end
end
