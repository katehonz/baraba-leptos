class UblExportService
  def self.export(invoice : Invoice, company : Company) : String
    counterpart = CounterpartQuery.new.id(invoice.counterpart_id).first
    lines = InvoiceLineQuery.new.invoice_id(invoice.id)
    type_code = invoice.uncl1001_code || case invoice.document_type
    when "CREDIT_NOTE" then "381"
    when "DEBIT_NOTE"  then "383"
    else                    "380"
    end
    String.build do |io|
      io << %(<?xml version="1.0" encoding="UTF-8"?>\n)
      io << %(<Invoice xmlns="urn:oasis:names:specification:ubl:schema:xsd:Invoice-2" )
      io << %(xmlns:cac="urn:oasis:names:specification:ubl:schema:xsd:CommonAggregateComponents-2" )
      io << %(xmlns:cbc="urn:oasis:names:specification:ubl:schema:xsd:CommonBasicComponents-2">\n)
      io << "  <cbc:CustomizationID>urn:cen.eu:en16931:2017</cbc:CustomizationID>\n"
      io << "  <cbc:ID>#{escape(invoice.invoice_number || "")}</cbc:ID>\n"
      io << "  <cbc:IssueDate>#{invoice.issue_date.to_s("%Y-%m-%d")}</cbc:IssueDate>\n"
      io << "  <cbc:InvoiceTypeCode>#{type_code}</cbc:InvoiceTypeCode>\n"
      io << "  <cbc:DocumentCurrencyCode>#{invoice.currency}</cbc:DocumentCurrencyCode>\n"
      if oid = invoice.original_invoice_id
        if orig = InvoiceQuery.new.id(oid).first?
          io << "  <cac:BillingReference><cac:InvoiceDocumentReference><cbc:ID>#{escape(orig.invoice_number || "")}</cbc:ID></cac:InvoiceDocumentReference></cac:BillingReference>\n"
        end
      end
      io << "  <cac:AccountingSupplierParty><cac:Party>\n"
      io << "    <cac:PartyName><cbc:Name>#{escape(company.name)}</cbc:Name></cac:PartyName>\n"
      io << "    <cac:PartyTaxScheme><cbc:CompanyID schemeID=\"9947\">#{escape(company.vat_number || company.eik)}</cbc:CompanyID><cac:TaxScheme><cbc:ID>VAT</cbc:ID></cac:TaxScheme></cac:PartyTaxScheme>\n"
      io << "    <cac:PartyLegalEntity><cbc:CompanyID schemeID=\"0195\">#{escape(company.eik)}</cbc:CompanyID></cac:PartyLegalEntity>\n"
      io << "  </cac:Party></cac:AccountingSupplierParty>\n"
      io << "  <cac:AccountingCustomerParty><cac:Party>\n"
      io << "    <cac:PartyName><cbc:Name>#{escape(counterpart.try(&.name) || "")}</cbc:Name></cac:PartyName>\n"
      io << "  </cac:Party></cac:AccountingCustomerParty>\n"
      n = 0
      lines.each do |line|
        n += 1
        io << "  <cac:InvoiceLine>\n"
        io << "    <cbc:ID>#{n}</cbc:ID>\n"
        io << "    <cbc:InvoicedQuantity unitCode=\"#{escape(line.unit_code)}\">#{line.quantity}</cbc:InvoicedQuantity>\n"
        io << "    <cbc:LineExtensionAmount currencyID=\"#{invoice.currency}\">#{line.line_total}</cbc:LineExtensionAmount>\n"
        io << "    <cac:Item><cbc:Name>#{escape(line.description)}</cbc:Name>"
        if line.vat_exemption_reason_id
          if r = VatExemptionReasonQuery.new.id(line.vat_exemption_reason_id.not_nil!).first?
            io << "<cac:ClassifiedTaxCategory><cbc:ID>#{escape(r.ubl_category_code || "E")}</cbc:ID>"
            io << "<cbc:Percent>#{line.vat_rate}</cbc:Percent>"
            io << "<cbc:TaxExemptionReasonCode>#{escape(r.ubl_exemption_code || "")}</cbc:TaxExemptionReasonCode>"
            io << "<cac:TaxScheme><cbc:ID>VAT</cbc:ID></cac:TaxScheme></cac:ClassifiedTaxCategory>"
          end
        else
          io << "<cac:ClassifiedTaxCategory><cbc:ID>S</cbc:ID><cbc:Percent>#{line.vat_rate}</cbc:Percent><cac:TaxScheme><cbc:ID>VAT</cbc:ID></cac:TaxScheme></cac:ClassifiedTaxCategory>"
        end
        io << "</cac:Item>\n"
        io << "    <cac:Price><cbc:PriceAmount currencyID=\"#{invoice.currency}\">#{line.unit_price}</cbc:PriceAmount></cac:Price>\n"
        io << "  </cac:InvoiceLine>\n"
      end
      io << "  <cac:LegalMonetaryTotal>\n"
      io << "    <cbc:LineExtensionAmount currencyID=\"#{invoice.currency}\">#{invoice.subtotal}</cbc:LineExtensionAmount>\n"
      io << "    <cbc:TaxExclusiveAmount currencyID=\"#{invoice.currency}\">#{invoice.subtotal}</cbc:TaxExclusiveAmount>\n"
      io << "    <cbc:TaxInclusiveAmount currencyID=\"#{invoice.currency}\">#{invoice.total_amount}</cbc:TaxInclusiveAmount>\n"
      io << "    <cbc:PayableAmount currencyID=\"#{invoice.currency}\">#{invoice.total_amount}</cbc:PayableAmount>\n"
      io << "  </cac:LegalMonetaryTotal>\n"
      io << "</Invoice>\n"
    end
  end

  private def self.escape(s : String) : String
    s.gsub("&", "&amp;").gsub("<", "&lt;").gsub(">", "&gt;").gsub("\"", "&quot;")
  end
end
