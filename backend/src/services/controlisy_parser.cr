require "xml"

# Parses Controlisy XML export files and imports data
class ControlisyParser
  # Parsed contractor data
  struct Contractor
    property ca_contractor_id : String
    property name : String
    property eik : String
    property vat_number : String
    property inside_number : String

    def initialize(@ca_contractor_id, @name, @eik, @vat_number, @inside_number)
    end
  end

  # Parsed accounting detail (debit or credit line)
  struct AccountingDetail
    property direction : String          # "Debit" or "Credit"
    property account_number : String
    property account_name : String
    property amount_bgn : BigDecimal
    property currency : String?
    property currency_amount : BigDecimal?
    property unit : String?
    property quantity : BigDecimal?
    property contractor_name : String?
    property contractor_eik : String?
    property contractor_vat_number : String?
    property account_item1 : String?
    property account_item2 : String?
    property account_item3 : String?
    property account_item4 : String?

    def initialize(
      @direction,
      @account_number,
      @account_name,
      @amount_bgn,
      @currency = nil,
      @currency_amount = nil,
      @unit = nil,
      @quantity = nil,
      @contractor_name = nil,
      @contractor_eik = nil,
      @contractor_vat_number = nil,
      @account_item1 = nil,
      @account_item2 = nil,
      @account_item3 = nil,
      @account_item4 = nil
    )
    end

    def debit?
      direction == "Debit"
    end

    def credit?
      direction == "Credit"
    end
  end

  # Parsed accounting entry (one debit + one credit)
  struct Accounting
    property amount_bgn : BigDecimal
    property ca_vat_operation_id : String?
    property details : Array(AccountingDetail)

    def initialize(@amount_bgn, @ca_vat_operation_id, @details)
    end

    def debit_detail
      details.find(&.debit?)
    end

    def credit_detail
      details.find(&.credit?)
    end
  end

  # Parsed VAT data
  struct VatEntry
    property tax_base : BigDecimal
    property vat_rate : BigDecimal
    property vat_amount_bgn : BigDecimal
    property ca_vat_operation_id : String?
    property vat_operation_iden : String?
    property vat_operation_iden_name : String?
    property vat_operation_name : String?
    property vat_operation_additional_iden : String?
    property vat_operation_additional_name : String?
    property vat_operation_detail : String?
    property vat_operation_ident1 : String?
    property vat_operation_ident2 : String?
    property vat_operation_ident3 : String?
    property vat_operation_ident4 : String?

    def initialize(
      @tax_base,
      @vat_rate,
      @vat_amount_bgn,
      @ca_vat_operation_id = nil,
      @vat_operation_iden = nil,
      @vat_operation_iden_name = nil,
      @vat_operation_name = nil,
      @vat_operation_additional_iden = nil,
      @vat_operation_additional_name = nil,
      @vat_operation_detail = nil,
      @vat_operation_ident1 = nil,
      @vat_operation_ident2 = nil,
      @vat_operation_ident3 = nil,
      @vat_operation_ident4 = nil
    )
    end
  end

  # Parsed VAT data section
  struct VatData
    property vat_register : String?      # "1" = purchases, "2" = sales
    property contractor_name : String?
    property contractor_vat_number : String?
    property entries : Array(VatEntry)

    def initialize(@vat_register, @contractor_name, @contractor_vat_number, @entries)
    end

    def purchases?
      vat_register == "1"
    end

    def sales?
      vat_register == "2"
    end
  end

  # Parsed related document
  struct RelDoc
    property ca_rel_document_id : String?
    property document_number : String?
    property document_date : String?
    property contractor_name : String?
    property contractor_eik : String?
    property contractor_vat_number : String?

    def initialize(
      @ca_rel_document_id = nil,
      @document_number = nil,
      @document_date = nil,
      @contractor_name = nil,
      @contractor_eik = nil,
      @contractor_vat_number = nil
    )
    end
  end

  # Parsed document
  struct Document
    property accounting_month : String
    property vat_month : String
    property maturity : String?
    property doc_ident : String?
    property document_date : String
    property document_number : String
    property ca_doc_id : String
    property reason : String?
    property remark : String?
    property net_amount_bgn : BigDecimal
    property vat_amount_bgn : BigDecimal
    property total_amount_bgn : BigDecimal
    property ca_vat_operation_id : String?
    property ca_doc_type_id : String?
    property ca_contractor_id : String?
    property accountings : Array(Accounting)
    property vat_data : VatData?
    property rel_doc : RelDoc?

    def initialize(
      @accounting_month,
      @vat_month,
      @document_date,
      @document_number,
      @ca_doc_id,
      @net_amount_bgn,
      @vat_amount_bgn,
      @total_amount_bgn,
      @accountings,
      @maturity = nil,
      @doc_ident = nil,
      @reason = nil,
      @remark = nil,
      @ca_vat_operation_id = nil,
      @ca_doc_type_id = nil,
      @ca_contractor_id = nil,
      @vat_data = nil,
      @rel_doc = nil
    )
    end
  end

  # Result of parsing
  struct ParseResult
    property contractors : Array(Contractor)
    property documents : Array(Document)
    property import_type : String    # "purchases", "sales", "bank"

    def initialize(@contractors, @documents, @import_type)
    end
  end

  # Parse XML content and return structured data
  def self.parse(xml_content : String) : ParseResult
    doc = XML.parse(xml_content)
    root = doc.first_element_child

    raise "Invalid XML: no root element" unless root
    raise "Invalid XML: expected ExportedData root" unless root.name == "ExportedData"

    contractors = parse_contractors(root)
    documents = parse_documents(root)
    import_type = detect_import_type(documents)

    ParseResult.new(contractors, documents, import_type)
  end

  private def self.parse_contractors(root : XML::Node) : Array(Contractor)
    contractors = [] of Contractor

    root.xpath_nodes("//Contractors/Contractor").each do |node|
      contractors << Contractor.new(
        ca_contractor_id: node["ca_contractorId"]? || "",
        name: node["contractorName"]? || "",
        eik: node["contractorEIK"]? || "",
        vat_number: node["contractorVATNumber"]? || "",
        inside_number: node["contractorInsideNumber"]? || ""
      )
    end

    contractors
  end

  private def self.parse_documents(root : XML::Node) : Array(Document)
    documents = [] of Document

    root.xpath_nodes("//Documents/Document").each do |node|
      accountings = parse_accountings(node)
      vat_data = parse_vat_data(node)
      rel_doc = parse_rel_doc(node)

      documents << Document.new(
        accounting_month: node["accountingMonth"]? || "",
        vat_month: node["vatMonth"]? || "",
        document_date: node["documentDate"]? || "",
        document_number: node["documentNumber"]? || "",
        ca_doc_id: node["ca_docId"]? || "",
        net_amount_bgn: parse_decimal(node["netAmountBGN"]?),
        vat_amount_bgn: parse_decimal(node["vatAmountBGN"]?),
        total_amount_bgn: parse_decimal(node["totalAmountBGN"]?),
        accountings: accountings,
        maturity: node["maturity"]?,
        doc_ident: node["docIdent"]?,
        reason: node["reason"]?,
        remark: node["remark"]?,
        ca_vat_operation_id: node["ca_vatOperationID"]?,
        ca_doc_type_id: node["ca_docTypeID"]?,
        ca_contractor_id: node["ca_contractorId"]?,
        vat_data: vat_data,
        rel_doc: rel_doc
      )
    end

    documents
  end

  private def self.parse_accountings(doc_node : XML::Node) : Array(Accounting)
    accountings = [] of Accounting

    doc_node.xpath_nodes("Accountings/Accounting").each do |node|
      details = parse_accounting_details(node)
      amount = parse_decimal(node["amountBGN"]?)

      accountings << Accounting.new(
        amount_bgn: amount,
        ca_vat_operation_id: node["ca_vatOperationID"]?,
        details: details
      )
    end

    accountings
  end

  private def self.parse_accounting_details(accounting_node : XML::Node) : Array(AccountingDetail)
    details = [] of AccountingDetail

    accounting_node.xpath_nodes("AccountingDetail").each do |node|
      details << AccountingDetail.new(
        direction: node["direction"]? || "",
        account_number: node["accountNumber"]? || "",
        account_name: node["accountName"]? || "",
        amount_bgn: parse_decimal(accounting_node["amountBGN"]?),
        currency: empty_to_nil(node["currency"]?),
        currency_amount: parse_decimal_or_nil(node["currencyAmount"]?),
        unit: empty_to_nil(node["unit"]?),
        quantity: parse_decimal_or_nil(node["quantity"]?),
        contractor_name: empty_to_nil(node["contractorName"]?),
        contractor_eik: empty_to_nil(node["contractorEIK"]?),
        contractor_vat_number: empty_to_nil(node["contractorVATNumber"]?),
        account_item1: empty_to_nil(node["accountItem1"]?),
        account_item2: empty_to_nil(node["accountItem2"]?),
        account_item3: empty_to_nil(node["accountItem3"]?),
        account_item4: empty_to_nil(node["accountItem4"]?)
      )
    end

    details
  end

  private def self.parse_vat_data(doc_node : XML::Node) : VatData?
    vat_data_node = doc_node.xpath_node("VATData")
    return nil unless vat_data_node

    entries = [] of VatEntry
    vat_data_node.xpath_nodes("VAT").each do |node|
      entries << VatEntry.new(
        tax_base: parse_decimal(node["taxBase"]?),
        vat_rate: parse_decimal(node["vatRate"]?),
        vat_amount_bgn: parse_decimal(node["vatAmountBGN"]?),
        ca_vat_operation_id: node["ca_vatOperationID"]?,
        vat_operation_iden: node["vatOperationIden"]?,
        vat_operation_iden_name: node["vatOperationIdenName"]?,
        vat_operation_name: node["vatOperationName"]?,
        vat_operation_additional_iden: node["vatOperationAdditionalIden"]?,
        vat_operation_additional_name: node["vatOperationAdditionalName"]?,
        vat_operation_detail: node["vatOperationDetail"]?,
        vat_operation_ident1: node["vatOperationIdent1"]?,
        vat_operation_ident2: node["vatOperationIdent2"]?,
        vat_operation_ident3: node["vatOperationIdent3"]?,
        vat_operation_ident4: node["vatOperationIdent4"]?
      )
    end

    VatData.new(
      vat_register: vat_data_node["vatRegister"]?,
      contractor_name: vat_data_node["contractorName"]?,
      contractor_vat_number: vat_data_node["contractorVATNumber"]?,
      entries: entries
    )
  end

  private def self.parse_rel_doc(doc_node : XML::Node) : RelDoc?
    rel_doc_node = doc_node.xpath_node("RelDoc")
    return nil unless rel_doc_node

    RelDoc.new(
      ca_rel_document_id: rel_doc_node["ca_relDocumentId"]?,
      document_number: rel_doc_node["relDocumentNumber"]?,
      document_date: rel_doc_node["relDocumentDate"]?,
      contractor_name: rel_doc_node["contractorName"]?,
      contractor_eik: rel_doc_node["contractorEIK"]?,
      contractor_vat_number: rel_doc_node["contractorVATNumber"]?
    )
  end

  private def self.detect_import_type(documents : Array(Document)) : String
    # Check VAT register to determine type
    documents.each do |doc|
      if vat_data = doc.vat_data
        return "purchases" if vat_data.purchases?
        return "sales" if vat_data.sales?
      end
    end

    # Check account numbers to guess type
    documents.each do |doc|
      doc.accountings.each do |acc|
        acc.details.each do |detail|
          case detail.account_number
          when /^401/  # Suppliers
            return "purchases" if detail.credit?
            return "bank" if detail.debit?
          when /^411/  # Customers
            return "sales" if detail.debit?
            return "bank" if detail.credit?
          when /^50/   # Cash/Bank accounts
            return "bank"
          end
        end
      end
    end

    "bank"  # Default
  end

  private def self.parse_decimal(value : String?) : BigDecimal
    return BigDecimal.new(0) if value.nil? || value.empty?
    BigDecimal.new(value.gsub(",", "."))
  rescue
    BigDecimal.new(0)
  end

  private def self.parse_decimal_or_nil(value : String?) : BigDecimal?
    return nil if value.nil? || value.empty?
    BigDecimal.new(value.gsub(",", "."))
  rescue
    nil
  end

  private def self.empty_to_nil(value : String?) : String?
    return nil if value.nil? || value.empty?
    value
  end
end
