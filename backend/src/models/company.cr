class Company < BaseModel
  skip_schema_enforcer
  table do
    column name : String
    column eik : String
    column vat_number : String?
    column address : String?
    column city : String?
    column country : String?
    column post_code : String?
    column phone : String?
    column email : String?
    column website : String?

    column manager_name : String?
    column manager_eik : String?
    column manager_egn : String?

    column accountant_name : String?
    column accountant_egn : String?

    # Пълномощник (Authorized Person) for VAT submission
    column authorized_person_name : String?
    column authorized_person_egn : String?

    # Branch/Division number for companies with multiple branches
    column vat_branch_number : Int32?

    column tax_authority : String?  # TD на NAP
    column nap_office : String?

    column inventory_valuation_method : String = "FIFO"  # FIFO, LIFO, WEIGHTED_AVG
    column is_vat_registered : Bool = false
    column vat_period : String = "monthly"
    column currency : String = "EUR"
    column fiscal_year_start_month : Int32 = 1

    # SAF-T Header fields
    column tax_accounting_basis : String = "A"   # A, BANK, INSURANCE, P
    column tax_entity : String?                  # Branch/division name
    column software_company_name : String?
    column software_id : String?
    column software_version : String?

    # Ownership Structure (S.O.1-S.O.11)
    # 1=Headquarters, 2=Part of group, 3=Not part of group, 4=Solo proprietor, 5=Other
    column is_part_of_group : String?

    # Beneficial Owner (действителен собственик)
    column beneficial_owner_first_name_bg : String?
    column beneficial_owner_last_name_bg : String?
    column beneficial_owner_egn : String?
    column beneficial_owner_first_name_latin : String?
    column beneficial_owner_last_name_latin : String?
    column beneficial_owner_country : String?

    # Ultimate Parent (крайно предприятие майка)
    column ultimate_owner_name_bg : String?
    column ultimate_owner_uic : String?
    column ultimate_owner_name_latin : String?
    column ultimate_owner_country : String?

    # SAF-T address structure
    column street_name : String?
    column building_number : String?
    column region : String?                      # ISO 3166-2:BG

    # Settings stored as JSON string (parse at runtime)
    column settings : String = "{}"

    # Default Accounts (IDs)
    column cash_account_id : Int64?
    column bank_account_id : Int64?
    column customers_account_id : Int64?
    column suppliers_account_id : Int64?
    column revenues_account_id : Int64?
    column expenses_account_id : Int64?
    column delivery_buffer_account_id : Int64?
    column vat_receivable_account_id : Int64?
    column vat_payable_account_id : Int64?

    # Invoice Settings
    column invoice_next_number : Int64 = 1
    column representative_type : String?  # MANAGER, AUTHORIZED_PERSON

    # Ownership
    belongs_to owner : User

    has_many accounts : Account
    has_many counterparts : Counterpart
    has_many invoices : Invoice
    has_many journal_entries : JournalEntry
    has_many products : Product
    has_many bank_accounts : BankAccount
    has_many warehouses : Warehouse
    has_many stock_transactions : StockTransaction
    has_many fixed_asset_categories : FixedAssetCategory
    has_many fixed_assets : FixedAsset
    has_many fixed_asset_transactions : FixedAssetTransaction
    has_many accounting_periods : AccountingPeriod
    has_many documents : Document
    has_many opening_balances : OpeningBalance
    has_many shareholders : Shareholder
    has_many dividends : Dividend
    has_many vat_returns : VatReturn
    has_many dividend_distributions : DividendDistribution
    has_many scanned_invoices : ScannedInvoice
    has_many user_company_roles : UserCompanyRole
    has_many payments : Payment
    has_many physical_stocks : PhysicalStock
    has_many asset_transactions : AssetTransaction
    has_many beneficial_owners : BeneficialOwner
    has_many ultimate_parents : UltimateParent
    has_many company_locations : CompanyLocation
  end

  # Helper to parse settings JSON
  def parsed_settings : JSON::Any
    return JSON::Any.new({} of String => JSON::Any) if self.settings.empty?
    begin
      JSON.parse(self.settings)
    rescue
      JSON::Any.new({} of String => JSON::Any)
    end
  end

  # Helper methods to access settings
  def azure_di_endpoint
    parsed_settings.dig?("azure_di", "endpoint")
  end

  def azure_di_api_key
    parsed_settings.dig?("azure_di", "api_key")
  end

  def azure_di_enabled
    parsed_settings.dig?("azure_di", "enabled").try(&.as_bool?) || false
  end

  def saltedge_app_id
    parsed_settings.dig?("saltedge", "app_id")
  end

  def saltedge_secret
    parsed_settings.dig?("saltedge", "secret")
  end

  def saltedge_enabled
    parsed_settings.dig?("saltedge", "enabled").try(&.as_bool?) || false
  end

  def saltedge_customer_id
    parsed_settings.dig?("saltedge", "customer_id")
  end

  def wise_api_key
    parsed_settings.dig?("wise", "api_key")
  end

  def wise_profile_id
    parsed_settings.dig?("wise", "profile_id")
  end

  def wise_enabled
    parsed_settings.dig?("wise", "enabled").try(&.as_bool?) || false
  end

  def vies_validation_enabled
    parsed_settings.dig?("vies", "validation_enabled").try(&.as_bool?) || false
  end

  def s3_enabled
    parsed_settings.dig?("s3", "enabled").try(&.as_bool?) || false
  end

  def s3_bucket
    parsed_settings.dig?("s3", "bucket")
  end

  def s3_region
    parsed_settings.dig?("s3", "region").try(&.as_s?) || "eu-central-1"
  end

  def s3_endpoint
    parsed_settings.dig?("s3", "endpoint")
  end

  def s3_access_key
    parsed_settings.dig?("s3", "access_key")
  end

  def s3_secret_key
    parsed_settings.dig?("s3", "secret_key")
  end

  def s3_folder_prefix
    parsed_settings.dig?("s3", "folder_prefix")
  end

  # Mistral AI settings
  def mistral_api_key
    parsed_settings.dig?("mistral", "api_key")
  end

  def mistral_model
    parsed_settings.dig?("mistral", "model").try(&.as_s?) || "pixtral-12b-2409"
  end

  def mistral_enabled
    parsed_settings.dig?("mistral", "enabled").try(&.as_bool?) || false
  end

  # Helper to update settings
  def update_setting(key : String, value)
    current = parsed_settings.as_h? || {} of String => JSON::Any
    current[key] = JSON::Any.new(value)
    self.settings = current.to_json
  end
end
