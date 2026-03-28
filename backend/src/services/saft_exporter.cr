# SAF-T BG XML Exporter
# Generates SAF-T compliant XML files for Bulgarian tax reporting
# Based on BG_SAFT_Schema_V_1.0.1.xsd

require "xml"

class SaftExporter
  SAFT_VERSION = "1.0"
  SAFT_COUNTRY = "BG"
  DEFAULT_CURRENCY = "BGN"

  enum ReportType
    Monthly    # Месечно отчитане
    OnDemand   # При поискване - Материални запаси
    Annual     # Годишен отчет - Активи
  end

  getter company : Company
  getter period_start : Time
  getter period_end : Time
  getter report_type : ReportType

  def initialize(@company : Company, @period_start : Time, @period_end : Time, @report_type : ReportType = ReportType::Monthly)
  end

  # Generate the complete SAF-T XML
  def generate : String
    XML.build(indent: "  ", encoding: "UTF-8") do |xml|
      xml.element("AuditFile", {
        "xmlns" => "urn:StandardAuditFile-Taxation-Financial:BG",
        "xmlns:xsi" => "http://www.w3.org/2001/XMLSchema-instance"
      }) do
        build_header(xml)
        build_master_files(xml)
        build_general_ledger_entries(xml)
        build_source_documents(xml)
      end
    end
  end

  private def build_header(xml : XML::Builder)
    xml.element("Header") do
      xml.element("AuditFileVersion") { xml.text SAFT_VERSION }
      xml.element("AuditFileCountry") { xml.text SAFT_COUNTRY }
      xml.element("AuditFileDateCreated") { xml.text Time.utc.to_s("%Y-%m-%d") }
      xml.element("SoftwareCompanyName") { xml.text(@company.software_company_name || "Baraba") }
      xml.element("SoftwareID") { xml.text(@company.software_id || "BARABA") }
      xml.element("SoftwareVersion") { xml.text(@company.software_version || "2.0") }

      # Company info
      xml.element("Company") do
        xml.element("RegistrationNumber") { xml.text @company.eik }
        xml.element("Name") { xml.text @company.name }

        # Use main location if available, otherwise fallback to company address
        main_location = CompanyLocationQuery.new.company_id(@company.id).is_main(true).is_active(true).first?
        if main_location
          build_location_address(xml, main_location)
        else
          build_address(xml, @company)
        end

        build_contact(xml, @company)

        if @company.vat_number
          xml.element("TaxRegistration") do
            xml.element("TaxRegistrationNumber") { xml.text @company.vat_number.not_nil! }
            xml.element("TaxType") { xml.text "100" } # VAT
            xml.element("TaxAuthority") { xml.text(@company.tax_authority || "НАП") }
          end
        end

        # Bank accounts
        build_bank_accounts(xml)
      end

      # Ownership Structure
      build_ownership_structure(xml)

      xml.element("DefaultCurrencyCode") { xml.text(@company.currency || DEFAULT_CURRENCY) }

      # Selection criteria
      xml.element("SelectionCriteria") do
        xml.element("SelectionStartDate") { xml.text @period_start.to_s("%Y-%m-%d") }
        xml.element("SelectionEndDate") { xml.text @period_end.to_s("%Y-%m-%d") }
      end

      xml.element("TaxAccountingBasis") { xml.text(@company.tax_accounting_basis || "A") }
    end
  end

  private def build_location_address(xml : XML::Builder, location : CompanyLocation)
    xml.element("Address") do
      xml.element("StreetName") { xml.text location.street_name.not_nil! } if location.street_name
      xml.element("BuildingIdentifier") { xml.text location.building_number.not_nil! } if location.building_number
      xml.element("City") { xml.text location.city.not_nil! } if location.city
      xml.element("PostalCode") { xml.text location.post_code.not_nil! } if location.post_code
      xml.element("Region") { xml.text location.region.not_nil! } if location.region
      xml.element("Country") { xml.text location.country }
    end
  end

  private def build_address(xml : XML::Builder, entity)
    xml.element("Address") do
      # For SAFt structured address: use street_name if available, otherwise fallback to address
      street = if entity.responds_to?(:street_name)
                 entity.street_name.try { |s| s.empty? ? nil : s }
               else
                 nil
               end
      street ||= if entity.responds_to?(:address)
                   entity.address.try { |s| s.empty? ? nil : s }
                 else
                   nil
                 end
      xml.element("StreetName") { xml.text street } if street

      if entity.responds_to?(:building_number)
        bn = entity.building_number.try { |s| s.empty? ? nil : s }
        xml.element("BuildingIdentifier") { xml.text bn } if bn
      end

      if entity.responds_to?(:city)
        city = entity.city.try { |s| s.empty? ? nil : s }
        xml.element("City") { xml.text city } if city
      end

      if entity.responds_to?(:post_code)
        pc = entity.post_code.try { |s| s.empty? ? nil : s }
        xml.element("PostalCode") { xml.text pc } if pc
      end

      if entity.responds_to?(:region)
        reg = entity.region.try { |s| s.empty? ? nil : s }
        xml.element("Region") { xml.text reg } if reg
      end

      country = entity.responds_to?(:country_code) ? entity.country_code : entity.country
      xml.element("Country") { xml.text(country || "BG") }
    end
  end

  private def build_contact(xml : XML::Builder, entity)
    xml.element("Contact") do
      # Try contact_person first, then manager_name for Company
      contact_name = if entity.responds_to?(:contact_person)
                       entity.contact_person.try { |s| s.empty? ? nil : s }
                     else
                       nil
                     end
      contact_name ||= if entity.responds_to?(:manager_name)
                         entity.manager_name.try { |s| s.empty? ? nil : s }
                       else
                         nil
                       end

      if contact_name
        parts = contact_name.split(" ", 2)
        xml.element("ContactPerson") do
          xml.element("FirstName") { xml.text parts.first }
          xml.element("LastName") { xml.text(parts.size > 1 ? parts.last : parts.first) }
        end
      end

      if entity.responds_to?(:phone)
        phone = entity.phone.try { |s| s.empty? ? nil : s }
        xml.element("Telephone") { xml.text phone } if phone
      end

      if entity.responds_to?(:email)
        email = entity.email.try { |s| s.empty? ? nil : s }
        xml.element("Email") { xml.text email } if email
      end
    end
  end

  private def build_bank_accounts(xml : XML::Builder)
    bank_accounts = BankAccountQuery.new.company_id(@company.id)
    bank_accounts.each do |ba|
      xml.element("BankAccount") do
        xml.element("IBANNumber") { xml.text ba.iban }
        xml.element("BankAccountName") { xml.text ba.name }
        xml.element("BIC") { xml.text ba.bic.not_nil! } if ba.bic
        xml.element("CurrencyCode") { xml.text(ba.currency || DEFAULT_CURRENCY) }
      end
    end
  end

  private def build_ownership_structure(xml : XML::Builder)
    return unless @company.is_part_of_group

    beneficial_owners = BeneficialOwnerQuery.new.company_id(@company.id).is_active(true)
    ultimate_parents = UltimateParentQuery.new.company_id(@company.id).is_active(true)

    # Skip if no ownership data
    return if beneficial_owners.none? && ultimate_parents.none? && @company.is_part_of_group.try(&.empty?)

    xml.element("OwnershipStructure") do
      xml.element("IsPartOfGroup") { xml.text @company.is_part_of_group.not_nil! }

      # Multiple beneficial owners (S.O.3-S.O.8)
      beneficial_owners.each do |owner|
        xml.element("BeneficialOwner") do
          xml.element("BeneficialOwnerFirstNameBG") { xml.text owner.first_name_bg }
          xml.element("BeneficialOwnerLastNameBG") { xml.text owner.last_name_bg }
          xml.element("BeneficialOwnerEGN") { xml.text owner.egn.not_nil! } if owner.egn
          xml.element("BeneficialOwnerFirstNameLatin") { xml.text owner.first_name_latin.not_nil! } if owner.first_name_latin
          xml.element("BeneficialOwnerLastNameLatin") { xml.text owner.last_name_latin.not_nil! } if owner.last_name_latin
          xml.element("BeneficialOwnerCountry") { xml.text owner.country }
        end
      end

      # Multiple ultimate parents (S.O.9-S.O.11)
      ultimate_parents.each do |parent|
        xml.element("UltimateParent") do
          xml.element("UltimateParentNameBG") { xml.text parent.name_bg }
          xml.element("UltimateParentUICBG") { xml.text parent.uic.not_nil! } if parent.uic
          xml.element("UltimateParentNameLatin") { xml.text parent.name_latin.not_nil! } if parent.name_latin
          xml.element("UltimateParentCountry") { xml.text parent.country }
        end
      end
    end
  end

  private def build_master_files(xml : XML::Builder)
    xml.element("MasterFiles") do
      build_general_ledger_accounts(xml)
      build_customers(xml)
      build_suppliers(xml)
      build_tax_table(xml)
      build_uom_table(xml)
      build_products(xml)

      # On-demand: Physical Stock
      if @report_type == ReportType::OnDemand
        build_physical_stock(xml)
      end

      # Annual: Assets
      if @report_type == ReportType::Annual
        build_assets(xml)
      end
    end
  end

  private def build_general_ledger_accounts(xml : XML::Builder)
    xml.element("GeneralLedgerAccounts") do
      accounts = AccountQuery.new.company_id(@company.id).is_active.eq(true)
      accounts.each do |account|
        xml.element("Account") do
          xml.element("AccountID") { xml.text account.saft_standard_account_id }
          xml.element("AccountDescription") { xml.text account.name }
          xml.element("StandardAccountID") { xml.text account.saft_account.try(&.code) || account.code }
          xml.element("AccountType") { xml.text map_account_type(account.account_type) }

          xml.element("OpeningDebitBalance") { xml.text format_amount(account.opening_debit_balance) }
          xml.element("OpeningCreditBalance") { xml.text format_amount(account.opening_credit_balance) }
          xml.element("ClosingDebitBalance") { xml.text format_amount(account.closing_debit_balance) }
          xml.element("ClosingCreditBalance") { xml.text format_amount(account.closing_credit_balance) }
        end
      end
    end
  end

  private def build_customers(xml : XML::Builder)
    xml.element("Customers") do
      counterparts = CounterpartQuery.new.company_id(@company.id).is_active.eq(true)
      counterparts.select { |c| c.counterpart_type == "customer" || c.counterpart_type == "both" }.each do |cp|
        xml.element("Customer") do
          xml.element("CustomerID") { xml.text cp.saft_id || generate_saft_id(cp) }
          xml.element("AccountID") { xml.text cp.account.try(&.saft_standard_account_id) || @company.customers_account_id.to_s }
          xml.element("CustomerName") { xml.text cp.name }
          build_address(xml, cp)
          build_contact(xml, cp)

          if cp.vat_number
            xml.element("CustomerTaxRegistration") do
              xml.element("TaxRegistrationNumber") { xml.text cp.vat_number.not_nil! }
              xml.element("TaxType") { xml.text "100" }
            end
          end

          xml.element("SelfBillingIndicator") { xml.text cp.self_billing_indicator ? "1" : "0" }

          if cp.is_related_party
            xml.element("RelatedParty") do
              xml.element("RelatedPartyIndicator") { xml.text "1" }
              xml.element("RelatedPartyStartDate") { xml.text cp.related_party_start_date.not_nil!.to_s("%Y-%m-%d") } if cp.related_party_start_date
              xml.element("RelatedPartyEndDate") { xml.text cp.related_party_end_date.not_nil!.to_s("%Y-%m-%d") } if cp.related_party_end_date
            end
          end

          xml.element("OpeningDebitBalance") { xml.text format_amount(cp.opening_debit_balance) }
          xml.element("OpeningCreditBalance") { xml.text format_amount(cp.opening_credit_balance) }
          xml.element("ClosingDebitBalance") { xml.text format_amount(cp.closing_debit_balance) }
          xml.element("ClosingCreditBalance") { xml.text format_amount(cp.closing_credit_balance) }
        end
      end
    end
  end

  private def build_suppliers(xml : XML::Builder)
    xml.element("Suppliers") do
      counterparts = CounterpartQuery.new.company_id(@company.id).is_active.eq(true)
      counterparts.select { |c| c.counterpart_type == "supplier" || c.counterpart_type == "both" }.each do |cp|
        xml.element("Supplier") do
          xml.element("SupplierID") { xml.text cp.saft_id || generate_saft_id(cp) }
          xml.element("AccountID") { xml.text cp.account.try(&.saft_standard_account_id) || @company.suppliers_account_id.to_s }
          xml.element("SupplierName") { xml.text cp.name }
          build_address(xml, cp)
          build_contact(xml, cp)

          if cp.vat_number
            xml.element("SupplierTaxRegistration") do
              xml.element("TaxRegistrationNumber") { xml.text cp.vat_number.not_nil! }
              xml.element("TaxType") { xml.text "100" }
            end
          end

          xml.element("SelfBillingIndicator") { xml.text cp.self_billing_indicator ? "1" : "0" }

          xml.element("OpeningDebitBalance") { xml.text format_amount(cp.opening_debit_balance) }
          xml.element("OpeningCreditBalance") { xml.text format_amount(cp.opening_credit_balance) }
          xml.element("ClosingDebitBalance") { xml.text format_amount(cp.closing_debit_balance) }
          xml.element("ClosingCreditBalance") { xml.text format_amount(cp.closing_credit_balance) }
        end
      end
    end
  end

  private def build_tax_table(xml : XML::Builder)
    xml.element("TaxTable") do
      xml.element("TaxTableEntry") do
        xml.element("TaxType") { xml.text "100" }
        xml.element("Description") { xml.text "ДДС" }

        # Standard rates
        [
          {"100211", "20.00", "ДО със ставка 20%"},
          {"100213", "9.00", "ДО със ставка 9%"},
          {"100214", "0.00", "ДО със ставка 0%"},
          {"100219", "0.00", "Освободени доставки"}
        ].each do |code, rate, desc|
          xml.element("TaxCodeDetails") do
            xml.element("TaxCode") { xml.text code }
            xml.element("Description") { xml.text desc }
            xml.element("TaxPercentage") { xml.text rate }
          end
        end
      end
    end
  end

  private def build_uom_table(xml : XML::Builder)
    xml.element("UOMTable") do
      [
        {"C62", "брой"},
        {"KGM", "килограм"},
        {"LTR", "литър"},
        {"MTR", "метър"},
        {"MTK", "кв.м."},
        {"HUR", "час"}
      ].each do |code, desc|
        xml.element("UOMTableEntry") do
          xml.element("UnitOfMeasure") { xml.text code }
          xml.element("Description") { xml.text desc }
        end
      end
    end
  end

  private def build_products(xml : XML::Builder)
    xml.element("Products") do
      products = ProductQuery.new.company_id(@company.id).is_active.eq(true)
      products.each do |product|
        xml.element("Product") do
          xml.element("ProductCode") { xml.text product.code }
          xml.element("ProductGroup") { xml.text product.product_group.not_nil! } if product.product_group
          xml.element("GoodsServicesID") { xml.text product.goods_services_id || "01" }
          xml.element("Description") { xml.text product.name }
          xml.element("ProductCommodityCode") { xml.text product.commodity_code.not_nil! } if product.commodity_code
          xml.element("ProductNumberCode") { xml.text product.product_number_code.not_nil! } if product.product_number_code
          xml.element("ValuationMethod") { xml.text product.valuation_method || @company.inventory_valuation_method }
          xml.element("UOMBase") { xml.text product.uom_base || product.measure_unit || "C62" }
          xml.element("UOMStandard") { xml.text product.uom_standard || "C62" }
          xml.element("UOMToUOMBaseConversionFactor") { xml.text format_decimal(product.uom_conversion_factor) }
          xml.element("ProductType") { xml.text product.product_type || "30" }
        end
      end
    end
  end

  private def build_physical_stock(xml : XML::Builder)
    xml.element("PhysicalStock") do
      stocks = PhysicalStockQuery.new.company_id(@company.id)
      stocks.each do |stock|
        xml.element("PhysicalStockEntry") do
          xml.element("WarehouseID") { xml.text stock.warehouse_id.to_s }
          xml.element("LocationID") { xml.text stock.location_id.not_nil! } if stock.location_id
          xml.element("ProductCode") { xml.text stock.product.code }
          xml.element("StockAccountNo") { xml.text stock.stock_account_no.not_nil! } if stock.stock_account_no
          xml.element("ProductType") { xml.text stock.product_type }
          xml.element("ProductStatus") { xml.text stock.product_status.not_nil! } if stock.product_status
          xml.element("OwnerID") { xml.text stock.owner_id }
          xml.element("UOMPhysicalStock") { xml.text stock.uom_physical_stock }
          xml.element("UnitPriceBegin") { xml.text format_amount(stock.unit_price_begin) }
          xml.element("UnitPriceEnd") { xml.text format_amount(stock.unit_price_end) }
          xml.element("OpeningStockQuantity") { xml.text format_decimal(stock.opening_stock_quantity) }
          xml.element("OpeningStockValue") { xml.text format_amount(stock.opening_stock_value) }
          xml.element("ClosingStockQuantity") { xml.text format_decimal(stock.closing_stock_quantity) }
          xml.element("ClosingStockValue") { xml.text format_amount(stock.closing_stock_value) }
        end
      end
    end
  end

  private def build_assets(xml : XML::Builder)
    xml.element("Assets") do
      assets = FixedAssetQuery.new.company_id(@company.id)
      assets.each do |asset|
        xml.element("Asset") do
          xml.element("AssetID") { xml.text asset.inventory_number }
          xml.element("AccountID") { xml.text asset.account.try(&.saft_standard_account_id) || asset.category.try(&.asset_account).try(&.saft_standard_account_id) || "" }
          xml.element("Description") { xml.text asset.name }
          xml.element("SupplierID") { xml.text asset.supplier.try(&.saft_id) || "0" }
          xml.element("DateOfAcquisition") { xml.text asset.acquisition_date.to_s("%Y-%m-%d") }
          xml.element("StartUpDate") { xml.text (asset.put_into_service_date || asset.acquisition_date).to_s("%Y-%m-%d") }

          # SAP - Счетоводен амортизационен план
          xml.element("Valuations") do
            xml.element("Valuation") do
              xml.element("AssetValuationType") { xml.text "SAP" }
              xml.element("ValuationClass") { xml.text asset.sap_valuation_class || "1" }
              xml.element("AcquisitionAndProductionCostsBegin") { xml.text format_amount(asset.acquisition_cost) }
              xml.element("AcquisitionAndProductionCostsEnd") { xml.text format_amount(asset.acquisition_cost - asset.sap_asset_disposal) }
              xml.element("InvestmentSupport") { xml.text format_amount(asset.sap_investment_support) }
              xml.element("AssetLifeYear") { xml.text asset.sap_asset_life_years.to_s } if asset.sap_asset_life_years
              xml.element("DepreciationMethod") { xml.text asset.depreciation_method }
              xml.element("DepreciationPercentage") { xml.text format_decimal(asset.accounting_depreciation_rate) }
              xml.element("DepreciationForPeriod") { xml.text format_amount(asset.calculate_sap_depreciation) }
              xml.element("AccumulatedDepreciation") { xml.text format_amount(asset.sap_accumulated_depreciation) }
              xml.element("BookValueBegin") { xml.text format_amount(asset.sap_book_value_begin) }
              xml.element("BookValueEnd") { xml.text format_amount(asset.sap_book_value_end) }
            end
          end

          # DAP - Данъчен амортизационен план
          xml.element("Valuations") do
            xml.element("Valuation") do
              xml.element("AssetValuationType") { xml.text "DAP" }
              xml.element("ValuationClass") { xml.text asset.dap_valuation_class || "1" }
              xml.element("CategoryOfTaxDepreciableAsset") { xml.text asset.dap_category || "I" }
              xml.element("TaxDepreciableAssetValue") { xml.text format_amount(asset.dap_tax_depreciable_value) }
              xml.element("AccruedTaxDepreciation") { xml.text format_amount(asset.dap_accrued_tax_depreciation) }
              xml.element("TaxValueOfAsset") { xml.text format_amount(asset.dap_tax_value_begin) }
              xml.element("AnnualTaxDepreciationRate") { xml.text format_decimal(asset.dap_annual_tax_rate) }
              xml.element("MonthsDepreciation") { xml.text asset.dap_months_depreciation.to_s }
              xml.element("DepreciationForPeriod") { xml.text format_amount(asset.dap_depreciation_for_period) }
              xml.element("AccumulatedDepreciation") { xml.text format_amount(asset.dap_accumulated_depreciation) }
              xml.element("TaxValueEndPeriod") { xml.text format_amount(asset.dap_tax_value_end) }
            end
          end
        end
      end
    end
  end

  private def build_general_ledger_entries(xml : XML::Builder)
    entries = JournalEntryQuery.new.company_id(@company.id).status("posted")
      .entry_date.gte(@period_start).entry_date.lte(@period_end)

    total_debit = 0.0
    total_credit = 0.0
    entry_count = 0

    # Calculate totals first
    entries.each do |entry|
      lines = entry.parsed_lines.as_a?
      next unless lines
      lines.each do |line|
        total_debit += line["debit"]?.try(&.as_f?) || 0.0
        total_credit += line["credit"]?.try(&.as_f?) || 0.0
      end
      entry_count += 1
    end

    xml.element("GeneralLedgerEntries") do
      xml.element("NumberOfEntries") { xml.text entry_count.to_s }
      xml.element("TotalDebit") { xml.text format_amount(total_debit) }
      xml.element("TotalCredit") { xml.text format_amount(total_credit) }

      xml.element("Journal") do
        xml.element("JournalID") { xml.text "GL" }
        xml.element("Description") { xml.text "Главна книга" }
        xml.element("Type") { xml.text "GL" }

        entries.each do |entry|
          build_transaction(xml, entry)
        end
      end
    end
  end

  private def build_transaction(xml : XML::Builder, entry : JournalEntry)
    xml.element("Transaction") do
      xml.element("TransactionID") { xml.text entry.transaction_id || "JE-#{entry.id}" }
      xml.element("Period") { xml.text (entry.period || entry.entry_date.month).to_s }
      xml.element("PeriodYear") { xml.text (entry.period_year || entry.entry_date.year).to_s }
      xml.element("TransactionDate") { xml.text entry.entry_date.to_s("%Y-%m-%d") }
      xml.element("SourceID") { xml.text entry.source_id || entry.user_id.to_s }
      xml.element("TransactionType") { xml.text entry.transaction_type || "N" }
      xml.element("Description") { xml.text entry.description }
      xml.element("BatchID") { xml.text entry.batch_id.not_nil! } if entry.batch_id
      xml.element("SystemEntryDate") { xml.text (entry.system_entry_date || entry.created_at).to_s("%Y-%m-%d") }
      xml.element("GLPostingDate") { xml.text (entry.gl_posting_date || entry.entry_date).to_s("%Y-%m-%d") }
      xml.element("CustomerID") { xml.text entry.saft_customer_id || company_saft_id }
      xml.element("SupplierID") { xml.text entry.saft_supplier_id || company_saft_id }

      lines = entry.parsed_lines.as_a?
      return unless lines

      line_num = 0
      lines.each do |line|
        line_num += 1
        xml.element("Line") do
          xml.element("RecordID") { xml.text "#{entry.id}-#{line_num}" }
          xml.element("AccountID") { xml.text line["account_code"]?.try(&.as_s) || line["account_id"]?.try(&.as_s) || "" }
          xml.element("CustomerID") { xml.text line["customer_id"]?.try(&.as_s) || entry.saft_customer_id || company_saft_id }
          xml.element("SupplierID") { xml.text line["supplier_id"]?.try(&.as_s) || entry.saft_supplier_id || company_saft_id }
          xml.element("Description") { xml.text line["description"]?.try(&.as_s) || entry.description }

          debit = line["debit"]?.try(&.as_f?) || 0.0
          credit = line["credit"]?.try(&.as_f?) || 0.0

          if debit > 0
            xml.element("DebitAmount") do
              xml.element("Amount") { xml.text format_amount(debit) }
              xml.element("CurrencyCode") { xml.text @company.currency || DEFAULT_CURRENCY }
              xml.element("CurrencyAmount") { xml.text format_amount(debit) }
              xml.element("ExchangeRate") { xml.text "1.00" }
            end
          end

          if credit > 0
            xml.element("CreditAmount") do
              xml.element("Amount") { xml.text format_amount(credit) }
              xml.element("CurrencyCode") { xml.text @company.currency || DEFAULT_CURRENCY }
              xml.element("CurrencyAmount") { xml.text format_amount(credit) }
              xml.element("ExchangeRate") { xml.text "1.00" }
            end
          end

          # Tax information
          if tax_type = line["tax_type"]?.try(&.as_s)
            xml.element("TaxInformation") do
              xml.element("TaxType") { xml.text tax_type }
              xml.element("TaxCode") { xml.text line["tax_code"]?.try(&.as_s) || "000000" }
              xml.element("TaxPercentage") { xml.text format_decimal(line["tax_percentage"]?.try(&.as_f?) || 0.0) }
              xml.element("TaxBase") { xml.text format_amount(line["tax_base"]?.try(&.as_f?) || 0.0) }
              xml.element("TaxAmount") do
                xml.element("Amount") { xml.text format_amount(line["tax_amount"]?.try(&.as_f?) || 0.0) }
              end
            end
          end
        end
      end
    end
  end

  private def build_source_documents(xml : XML::Builder)
    xml.element("SourceDocuments") do
      build_sales_invoices(xml)
      build_purchase_invoices(xml)
      build_payments(xml)

      if @report_type == ReportType::OnDemand
        build_movement_of_goods(xml)
      end

      if @report_type == ReportType::Annual
        build_asset_transactions(xml)
      end
    end
  end

  private def build_sales_invoices(xml : XML::Builder)
    invoices = InvoiceQuery.new.company_id(@company.id).status("posted")
      .issue_date.gte(@period_start).issue_date.lte(@period_end)
    sales = invoices.select { |i| ["INVOICE", "DEBIT_NOTE", "CREDIT_NOTE"].includes?(i.document_type) }

    # Calculate totals
    total_debit = 0.0
    total_credit = 0.0
    entry_count = 0

    sales.each do |inv|
      if inv.document_type == "CREDIT_NOTE"
        total_credit += inv.total_amount
      else
        total_debit += inv.total_amount
      end
      entry_count += 1
    end

    xml.element("SalesInvoices") do
      xml.element("NumberOfEntries") { xml.text entry_count.to_s }
      xml.element("TotalDebit") { xml.text format_amount(total_debit) }
      xml.element("TotalCredit") { xml.text format_amount(total_credit) }

      sales.each do |invoice|
        build_invoice(xml, invoice, "Sales")
      end
    end
  end

  private def build_purchase_invoices(xml : XML::Builder)
    # For now, skip purchase invoices - they would come from scanned_invoices or similar
    xml.element("PurchaseInvoices") do
      xml.element("NumberOfEntries") { xml.text "0" }
      xml.element("TotalDebit") { xml.text "0.00" }
      xml.element("TotalCredit") { xml.text "0.00" }
    end
  end

  private def build_invoice(xml : XML::Builder, invoice : Invoice, type : String)
    xml.element("Invoice") do
      xml.element("InvoiceNo") { xml.text invoice.invoice_number }
      xml.element("InvoiceType") { xml.text invoice.saft_invoice_type || map_invoice_type(invoice.document_type) }
      xml.element("Period") { xml.text (invoice.period || invoice.issue_date.month).to_s }
      xml.element("PeriodYear") { xml.text (invoice.period_year || invoice.issue_date.year).to_s }
      xml.element("InvoiceDate") { xml.text invoice.issue_date.to_s("%Y-%m-%d") }
      xml.element("TransactionID") { xml.text invoice.transaction_id || "INV-#{invoice.id}" }
      xml.element("CustomerID") { xml.text invoice.saft_customer_id || invoice.counterpart.saft_id || "0" }
      xml.element("SupplierID") { xml.text invoice.saft_supplier_id || company_saft_id }
      xml.element("TaxPointDate") { xml.text (invoice.tax_event_date || invoice.issue_date).to_s("%Y-%m-%d") }
      xml.element("InvoiceStatus") { xml.text invoice.status.upcase }

      # Invoice lines
      lines = invoice.parsed_lines.as_a?
      if lines
        line_num = 0
        lines.each do |line|
          line_num += 1
          xml.element("InvoiceLine") do
            xml.element("LineNumber") { xml.text line_num.to_s }
            xml.element("ProductCode") { xml.text line["product_code"]?.try(&.as_s) || "" }
            xml.element("ProductDescription") { xml.text line["description"]?.try(&.as_s) || "" }
            xml.element("Quantity") { xml.text format_decimal(line["quantity"]?.try(&.as_f?) || 1.0) }
            xml.element("UnitOfMeasure") { xml.text line["unit"]?.try(&.as_s) || "C62" }
            xml.element("UnitPrice") { xml.text format_amount(line["unit_price"]?.try(&.as_f?) || 0.0) }
            xml.element("TaxPointDate") { xml.text (invoice.tax_event_date || invoice.issue_date).to_s("%Y-%m-%d") }
            xml.element("DebitCreditIndicator") { xml.text invoice.document_type == "CREDIT_NOTE" ? "C" : "D" }

            xml.element("InvoiceLineAmount") do
              xml.element("Amount") { xml.text format_amount(line["total"]?.try(&.as_f?) || 0.0) }
              xml.element("CurrencyCode") { xml.text invoice.currency || DEFAULT_CURRENCY }
            end

            xml.element("TaxInformation") do
              xml.element("TaxType") { xml.text "100" }
              xml.element("TaxCode") { xml.text line["tax_code"]?.try(&.as_s) || "100211" }
              xml.element("TaxPercentage") { xml.text format_decimal(line["vat_rate"]?.try(&.as_f?) || 20.0) }
              xml.element("TaxBase") { xml.text format_amount(line["subtotal"]?.try(&.as_f?) || 0.0) }
              xml.element("TaxAmount") do
                xml.element("Amount") { xml.text format_amount(line["vat_amount"]?.try(&.as_f?) || 0.0) }
              end
            end
          end
        end
      end

      # Document totals
      xml.element("DocumentTotals") do
        xml.element("TaxInformationTotals") do
          xml.element("TaxType") { xml.text "100" }
          xml.element("TaxCode") { xml.text "100211" }
          xml.element("TaxBase") { xml.text format_amount(invoice.subtotal) }
          xml.element("TaxAmount") do
            xml.element("Amount") { xml.text format_amount(invoice.vat_amount) }
          end
        end
        xml.element("NetTotal") { xml.text format_amount(invoice.subtotal) }
        xml.element("GrossTotal") { xml.text format_amount(invoice.total_amount) }
      end
    end
  end

  private def build_payments(xml : XML::Builder)
    payments = PaymentQuery.new.company_id(@company.id)
      .transaction_date.gte(@period_start).transaction_date.lte(@period_end)

    total_debit = 0.0
    total_credit = 0.0
    entry_count = 0

    payments.each do |p|
      # Calculate from lines
      PaymentLineQuery.new.payment_id(p.id).each do |line|
        if line.debit_credit_indicator == "D"
          total_debit += line.amount
        else
          total_credit += line.amount
        end
      end
      entry_count += 1
    end

    xml.element("Payments") do
      xml.element("NumberOfEntries") { xml.text entry_count.to_s }
      xml.element("TotalDebit") { xml.text format_amount(total_debit) }
      xml.element("TotalCredit") { xml.text format_amount(total_credit) }

      payments.each do |payment|
        xml.element("Payment") do
          xml.element("PaymentRefNo") { xml.text payment.payment_ref_no }
          xml.element("Period") { xml.text (payment.period || payment.transaction_date.month).to_s }
          xml.element("PeriodYear") { xml.text (payment.period_year || payment.transaction_date.year).to_s }
          xml.element("TransactionID") { xml.text payment.transaction_id }
          xml.element("TransactionDate") { xml.text payment.transaction_date.to_s("%Y-%m-%d") }
          xml.element("PaymentMethod") { xml.text payment.payment_method }
          xml.element("Description") { xml.text payment.description }
          xml.element("BatchID") { xml.text payment.batch_id.not_nil! } if payment.batch_id
          xml.element("SystemID") { xml.text payment.system_id.not_nil! } if payment.system_id
          xml.element("SourceID") { xml.text payment.source_id.not_nil! } if payment.source_id

          # Payment lines
          PaymentLineQuery.new.payment_id(payment.id).each do |line|
            xml.element("Line") do
              xml.element("LineNumber") { xml.text line.line_number.not_nil!.to_s } if line.line_number
              xml.element("SourceDocumentID") { xml.text line.source_document_id.not_nil! } if line.source_document_id
              xml.element("AccountID") { xml.text line.account.try(&.saft_standard_account_id) || "" }
              xml.element("CustomerID") { xml.text line.saft_customer_id || payment.saft_customer_id || "0" }
              xml.element("SupplierID") { xml.text line.saft_supplier_id || payment.saft_supplier_id || "0" }
              xml.element("TaxPointDate") { xml.text line.tax_point_date.not_nil!.to_s("%Y-%m-%d") } if line.tax_point_date
              xml.element("Description") { xml.text line.description.not_nil! } if line.description
              xml.element("DebitCreditIndicator") { xml.text line.debit_credit_indicator }

              xml.element("PaymentLineAmount") do
                xml.element("Amount") { xml.text format_amount(line.amount) }
                xml.element("CurrencyCode") { xml.text line.currency_code }
                xml.element("CurrencyAmount") { xml.text format_amount(line.currency_amount || line.amount) }
                xml.element("ExchangeRate") { xml.text format_decimal(line.exchange_rate) }
              end

              if line.tax_type
                xml.element("TaxInformation") do
                  xml.element("TaxType") { xml.text line.tax_type.not_nil! }
                  xml.element("TaxCode") { xml.text line.tax_code || "000000" }
                  xml.element("TaxPercentage") { xml.text format_decimal(line.tax_percentage || 0.0) }
                  xml.element("TaxBase") { xml.text format_amount(line.tax_base || 0.0) }
                  xml.element("TaxAmount") do
                    xml.element("Amount") { xml.text format_amount(line.tax_amount || 0.0) }
                  end
                end
              end
            end
          end

          # Document totals
          xml.element("PaymentDocumentTotals") do
            xml.element("NetTotal") { xml.text format_amount(payment.net_total || 0.0) }
            xml.element("GrossTotal") { xml.text format_amount(payment.gross_total) }
          end
        end
      end
    end
  end

  private def build_movement_of_goods(xml : XML::Builder)
    movements = StockTransactionQuery.new.company_id(@company.id)
      .transaction_date.gte(@period_start).transaction_date.lte(@period_end)

    total_received = 0.0
    total_issued = 0.0
    entry_count = movements.size

    movements.each do |m|
      if m.is_incoming
        total_received += m.quantity
      else
        total_issued += m.quantity
      end
    end

    xml.element("MovementOfGoods") do
      xml.element("NumberOfMovementLines") { xml.text entry_count.to_s }
      xml.element("TotalQuantityReceived") { xml.text format_decimal(total_received) }
      xml.element("TotalQuantityIssued") { xml.text format_decimal(total_issued) }

      movements.each do |movement|
        xml.element("StockMovement") do
          xml.element("MovementReference") { xml.text movement.movement_reference || "SM-#{movement.id}" }
          xml.element("MovementDate") { xml.text movement.transaction_date.to_s("%Y-%m-%d") }
          xml.element("MovementPostingDate") { xml.text (movement.movement_posting_date || movement.transaction_date).to_s("%Y-%m-%d") }
          xml.element("TaxPointDate") { xml.text (movement.tax_point_date || movement.transaction_date).to_s("%Y-%m-%d") }
          xml.element("MovementType") { xml.text movement.movement_type || "180" }
          xml.element("SourceID") { xml.text movement.source_id || "" }
          xml.element("SystemID") { xml.text movement.system_id || movement.id.to_s }

          if movement.document_number
            xml.element("DocumentReference") do
              xml.element("DocumentType") { xml.text movement.document_type || "OTHER" }
              xml.element("DocumentNumber") { xml.text movement.document_number.not_nil! }
              xml.element("DocumentLine") { xml.text movement.document_line || "1" }
            end
          end

          xml.element("Line") do
            xml.element("LineNumber") { xml.text (movement.line_number || 1).to_s }
            xml.element("AccountID") { xml.text movement.account.try(&.saft_standard_account_id) || "" }
            xml.element("TransactionID") { xml.text movement.transaction_id || "" }
            xml.element("CustomerID") { xml.text movement.saft_customer_id || company_saft_id }
            xml.element("SupplierID") { xml.text movement.saft_supplier_id || company_saft_id }
            xml.element("ProductCode") { xml.text movement.product.code }
            xml.element("StockAccountNo") { xml.text movement.stock_account_no.not_nil! } if movement.stock_account_no
            xml.element("Quantity") { xml.text format_decimal(movement.saft_quantity) }
            xml.element("UnitOfMeasure") { xml.text movement.unit_of_measure || "C62" }
            xml.element("UOMToUOMPhysicalStockConversion") { xml.text format_decimal(movement.uom_conversion_factor) }
            xml.element("BookValue") { xml.text format_amount(movement.book_value || movement.total_amount) }
            xml.element("MovementSubtype") { xml.text movement.movement_subtype.not_nil! } if movement.movement_subtype
            xml.element("MovementComments") { xml.text movement.movement_comments.not_nil! } if movement.movement_comments

            if movement.tax_type
              xml.element("TaxInformation") do
                xml.element("TaxType") { xml.text movement.tax_type.not_nil! }
                xml.element("TaxCode") { xml.text movement.tax_code || "000000" }
              end
            end
          end
        end
      end
    end
  end

  private def build_asset_transactions(xml : XML::Builder)
    transactions = AssetTransactionQuery.new.company_id(@company.id).in_year(@period_start.year)

    xml.element("AssetTransactions") do
      xml.element("NumberOfAssetTransactions") { xml.text transactions.size.to_s }

      transactions.each do |tx|
        xml.element("AssetTransaction") do
          xml.element("AssetTransactionID") { xml.text tx.asset_transaction_id }
          xml.element("AssetID") { xml.text tx.fixed_asset.inventory_number }
          xml.element("AssetTransactionType") { xml.text tx.asset_transaction_type }
          xml.element("Description") { xml.text tx.description.not_nil! } if tx.description
          xml.element("AssetTransactionDate") { xml.text tx.transaction_date.to_s("%Y-%m-%d") }

          if tx.counterpart
            xml.element("Supplier") do
              xml.element("SupplierName") { xml.text tx.counterpart.not_nil!.name }
              xml.element("SupplierID") { xml.text tx.saft_supplier_id || tx.counterpart.not_nil!.saft_id || "0" }
              build_address(xml, tx.counterpart.not_nil!)
            end
          end

          xml.element("TransactionID") { xml.text tx.transaction_id || "" }

          xml.element("AssetTransactionValuations") do
            xml.element("AssetTransactionValuation") do
              xml.element("AssetValuationType") { xml.text tx.valuation_type || "SAP" }
              xml.element("AcquisitionAndProductionCostOnTransaction") { xml.text format_amount(tx.acquisition_cost_on_transaction || 0.0) }
              xml.element("BookValueOnTransaction") { xml.text format_amount(tx.book_value_on_transaction) }
              xml.element("AssetTransactionAmount") { xml.text format_amount(tx.transaction_amount) }
            end
          end
        end
      end
    end
  end

  # Helper methods

  private def company_saft_id : String
    "10#{@company.eik}"
  end

  private def generate_saft_id(counterpart : Counterpart) : String
    if counterpart.eik && (counterpart.country_code.nil? || counterpart.country_code == "BG")
      "10#{counterpart.eik}"
    elsif counterpart.vat_number && counterpart.country_code
      cc = counterpart.country_code.not_nil!
      if Counterpart::EU_COUNTRIES.includes?(cc) && cc != "BG"
        "11#{cc}#{counterpart.vat_number}"
      else
        "12#{cc}#{counterpart.vat_number || counterpart.eik}"
      end
    elsif counterpart.eik && counterpart.eik.not_nil!.size == 10
      "13#{counterpart.eik}"
    else
      "15#{counterpart.id}"
    end
  end

  private def map_account_type(type : String) : String
    case type.downcase
    when "active", "asset"     then "GL"
    when "passive", "liability" then "GL"
    when "bifunctional"        then "GL"
    when "expense"             then "E"
    when "revenue", "income"   then "I"
    else                       "GL"
    end
  end

  private def map_invoice_type(doc_type : String) : String
    case doc_type.upcase
    when "INVOICE"     then "1"
    when "DEBIT_NOTE"  then "2"
    when "CREDIT_NOTE" then "3"
    else               "9"
    end
  end

  private def format_amount(amount : Float64?) : String
    return "0.00" if amount.nil?
    "%.2f" % amount
  end

  private def format_decimal(value : Float64?) : String
    return "1.00" if value.nil?
    "%.2f" % value
  end
end
