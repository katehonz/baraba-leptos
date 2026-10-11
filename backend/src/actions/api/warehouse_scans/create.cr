require "random/secure"

class Api::WarehouseScans::Create < ApiAction
  include Api::WarehouseScanJson

  accepted_formats [:json, :multipart_form], default: :json

  post "/api/companies/:company_id/warehouse_scans" do
    cid = company_id.to_i64
    authorize!(Permission::WAREHOUSE_CREATE, cid)
    company = CompanyQuery.new.find(cid)

    unless company.mistral_enabled
      response.status_code = 400
      return json({success: false, error: "Mistral не е включен за тази фирма"})
    end

    uploaded = params.get_file?("file")
    unless uploaded
      response.status_code = 400
      return json({success: false, error: "Качете файл като multipart поле file (без base64 JSON)"})
    end

    file_name = uploaded.filename.presence || "document.pdf"
    bytes = File.read(uploaded.path).to_slice
    if bytes.size > 20 * 1024 * 1024
      response.status_code = 422
      return json({success: false, error: "Файлът е над 20 MB"})
    end

    api_key = company.mistral_api_key.try(&.as_s?) || ENV["MISTRAL_API_KEY"]? || ""
    if api_key.empty?
      response.status_code = 400
      return json({success: false, error: "Моля конфигурирайте Mistral API ключ в настройките на фирмата"})
    end

    direction = params.get?("direction").try(&.to_s).presence || "inbound"
    direction = "inbound" unless WarehouseScan::DIRECTIONS.includes?(direction)

    stored_key = store_file(cid, file_name, bytes)

    begin
      parsed = MistralWarehouseService.scan(bytes, api_key, file_name, direction)
    rescue ex
      Log.error { "warehouse_scan.ocr_failed company=#{cid} error=#{ex.message}" }
      response.status_code = 422
      return json({success: false, error: "OCR грешка: #{ex.message}"})
    end

    counterpart_id = json_i64_param(params.get?("counterpart_id"))
    unless counterpart_id
      if vat = parsed.vendor_vat.try(&.presence)
        unless Counterpart.generic_eik?(vat)
          counterpart_id = CounterpartQuery.new.company_id(cid).vat_number(vat).first?.try(&.id)
          counterpart_id ||= Counterpart.find_by_unique_eik(cid, vat).try(&.id)
        end
      end
    end

    scan = SaveWarehouseScan.create!(
      company_id: cid,
      direction: direction,
      status: "pending",
      original_file_name: file_name,
      s3_key: stored_key,
      raw_json: parsed.raw_json,
      vendor_name: parsed.vendor_name,
      vendor_vat: parsed.vendor_vat,
      invoice_number: parsed.invoice_number,
      document_date: parsed.document_date,
      subtotal: parsed.subtotal,
      vat_amount: parsed.vat_amount,
      total_amount: parsed.total_amount,
      confidence: parsed.confidence,
      counterpart_id: counterpart_id
    )

    parsed.items.each_with_index do |item, i|
      mapping = VendorProductMappingService.find(cid, counterpart_id, item.sku, item.barcode, item.name)
      auto_pid = VendorProductMappingService.auto_product_id(mapping, item.confidence || parsed.confidence)
      status = if auto_pid
                 VendorProductMappingService.touch!(mapping.not_nil!) if mapping
                 "auto"
               else
                 "unmapped"
               end
      SaveWarehouseScanLine.create!(
        warehouse_scan_id: scan.id,
        raw_name: item.name,
        raw_sku: item.sku,
        raw_barcode: item.barcode,
        quantity: item.quantity,
        raw_unit: item.unit,
        unit_code: MistralWarehouseService.guess_unit(item.unit),
        unit_price: item.unit_price,
        vat_rate: item.vat_rate,
        mapping_status: status,
        confidence: item.confidence || parsed.confidence,
        line_number: i + 1,
        product_id: auto_pid
      )
    end

    scan = refresh_scan_status(scan)
    json({success: true, data: scan_json(scan)})
  end

  private def store_file(company_id : Int64, file_name : String, bytes : Bytes) : String?
    safe = file_name.gsub(/[^\w.\-]/, "_")[0, 80]
    rel = "warehouse-scans/#{company_id}/#{Random::Secure.hex(8)}_#{safe}"
    dir = File.join("uploads", "warehouse-scans", company_id.to_s)
    begin
      Dir.mkdir_p(dir)
      File.write(File.join("uploads", rel), bytes)
      rel
    rescue
      nil
    end
  end

  private def json_i64_param(val : String?) : Int64?
    return nil unless val.try(&.presence)
    val.not_nil!.to_i64?
  end
end
