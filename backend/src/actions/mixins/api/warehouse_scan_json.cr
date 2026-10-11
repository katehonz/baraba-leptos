module Api::WarehouseScanJson
  private def scan_json(scan : WarehouseScan)
    lines = WarehouseScanLineQuery.new.warehouse_scan_id(scan.id).line_number.asc_order
    {
      id:                     scan.id,
      direction:              scan.direction,
      status:                 scan.status,
      original_file_name:     scan.original_file_name,
      vendor_name:            scan.vendor_name,
      vendor_vat:             scan.vendor_vat,
      invoice_number:         scan.invoice_number,
      document_date:          scan.document_date.try(&.to_s("%Y-%m-%d")),
      subtotal:               scan.subtotal,
      vat_amount:             scan.vat_amount,
      total_amount:           scan.total_amount,
      confidence:             scan.confidence,
      notes:                  scan.notes,
      counterpart_id:         scan.counterpart_id,
      warehouse_document_id:  scan.warehouse_document_id,
      invoice_id:             scan.invoice_id,
      unmapped_count:         WarehouseScanLineQuery.new.warehouse_scan_id(scan.id).mapping_status("unmapped").select_count,
      lines:                  lines.map { |l| line_json(l) },
    }
  end

  private def line_json(l : WarehouseScanLine)
    product = l.product_id.try { |pid| ProductQuery.new.id(pid).first? }
    {
      id:             l.id,
      line_number:    l.line_number,
      raw_name:       l.raw_name,
      raw_sku:        l.raw_sku,
      raw_barcode:    l.raw_barcode,
      quantity:       l.quantity,
      raw_unit:       l.raw_unit,
      unit_code:      l.unit_code,
      unit_price:     l.unit_price,
      vat_rate:       l.vat_rate,
      mapping_status: l.mapping_status,
      confidence:     l.confidence,
      product_id:     l.product_id,
      product_code:   product.try(&.code),
      product_name:   product.try(&.name),
    }
  end

  private def refresh_scan_status(scan : WarehouseScan) : WarehouseScan
    return scan if scan.status == "applied" || scan.status == "rejected"
    unmapped = WarehouseScanLineQuery.new.warehouse_scan_id(scan.id).mapping_status("unmapped").select_count
    new_status = unmapped == 0 ? "reviewed" : "pending"
    if new_status != scan.status
      return SaveWarehouseScan.update!(scan, status: new_status)
    end
    scan
  end
end
