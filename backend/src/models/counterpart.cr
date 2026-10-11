class Counterpart < BaseModel
  table do
    column name : String
    column eik : String?                 # Bulgarian company registration number (ЕИК)
    column vat_number : String?          # VAT number (ДДС номер)
    column address : String?
    column contact_person : String?
    column email : String?
    column phone : String?
    column counterpart_type : String     # "customer", "supplier", or "both"
    column is_active : Bool = true

    # SAF-T ID fields (CustomerID/SupplierID format)
    column saft_id_type : String?        # 10, 11, 12, 13, 14, 15, 16
    column saft_id : String?             # Generated SAF-T ID (e.g., 10123456789)

    # SAF-T Address Structure
    column city : String?
    column post_code : String?
    column country_code : String?        # ISO 3166-1 (e.g., BG, DE)
    column region : String?              # ISO 3166-2:BG (e.g., BG-22)
    column street_name : String?
    column building_number : String?
    column additional_address : String?

    # Related Party (свързано лице)
    column is_related_party : Bool = false
    column related_party_start_date : Time?
    column related_party_end_date : Time?

    # Opening/Closing Balances (per reporting period)
    column opening_debit_balance : Float64 = 0.0
    column opening_credit_balance : Float64 = 0.0
    column closing_debit_balance : Float64 = 0.0
    column closing_credit_balance : Float64 = 0.0

    # Self-billing indicator
    column self_billing_indicator : Bool = false

    belongs_to company : Company
    belongs_to account : Account?  # Default account for SAF-T AccountID
    has_many invoices : Invoice
    has_many payments : Payment
  end

  # Generate SAF-T ID based on EIK/VAT number and country
  # Format: prefix + identifier
  # 10 + ЕИК = Bulgarian companies
  # 11 + CountryCode + VAT = EU companies
  # 12 + CountryCode + ID = Non-EU companies
  # 13 + ЕГН = Bulgarian individuals
  # 14 + CountryCode + ID = Foreign with internal code
  # 15 + ID = Internal code only
  # 16 + 307... = NRA service number
  def generate_saft_id!
    if Counterpart.generic_eik?(self.eik)
      self.saft_id_type = "15"
      self.saft_id = "15#{self.id}"
    elsif self.eik && (self.country_code.nil? || self.country_code == "BG")
      # Bulgarian company with EIK
      self.saft_id_type = "10"
      self.saft_id = "10#{self.eik}"
    elsif self.vat_number && self.country_code
      cc = self.country_code.not_nil!
      if EU_COUNTRIES.includes?(cc) && cc != "BG"
        # EU company (not Bulgaria)
        self.saft_id_type = "11"
        self.saft_id = "11#{cc}#{self.vat_number}"
      else
        # Non-EU company
        self.saft_id_type = "12"
        self.saft_id = "12#{cc}#{self.vat_number || self.eik}"
      end
    elsif self.eik && self.eik.not_nil!.size == 10
      # Bulgarian individual (ЕГН)
      self.saft_id_type = "13"
      self.saft_id = "13#{self.eik}"
    else
      # Internal code
      self.saft_id_type = "15"
      self.saft_id = "15#{self.id}"
    end
  end

  EU_COUNTRIES = ["AT", "BE", "BG", "HR", "CY", "CZ", "DK", "EE", "FI", "FR",
                  "DE", "GR", "HU", "IE", "IT", "LV", "LT", "LU", "MT", "NL",
                  "PL", "PT", "RO", "SK", "SI", "ES", "SE"]

  # VIES/VAT prefixes (EL for Greece, XI for Northern Ireland)
  EU_VAT_PREFIXES = EU_COUNTRIES + ["EL", "XI"]

  # Placeholder EIK for non-EU / unknown third-country counterparts — these may repeat
  GENERIC_EIK_9  = "999999999"
  GENERIC_EIK_13 = "9999999999999"
  GENERIC_EIKS   = [GENERIC_EIK_9, GENERIC_EIK_13]

  def self.strip_eu_prefix(value : String) : String
    v = value.strip.upcase.gsub(/[\s.\-]/, "")
    return v if v.size <= 2

    prefix = v[0, 2]
    if prefix.chars.all?(&.ascii_letter?) && EU_VAT_PREFIXES.includes?(prefix)
      v[2..]
    else
      v
    end
  end

  def self.normalize_eik(value : String?) : String?
    return nil if value.nil?
    stripped = strip_eu_prefix(value)
    stripped.empty? ? nil : stripped
  end

  def self.generic_eik?(value : String?) : Bool
    normalized = normalize_eik(value)
    return false if normalized.nil?
    GENERIC_EIKS.includes?(normalized)
  end

  # Find an existing counterpart by EIK. Returns nil for blank/generic EIK
  # (9 or 13 nines) so unknown third-country parties are not collapsed.
  def self.find_by_unique_eik(company_id : Int64, eik : String?) : Counterpart?
    normalized = normalize_eik(eik)
    return nil if normalized.nil? || GENERIC_EIKS.includes?(normalized)
    CounterpartQuery.new.company_id(company_id).eik(normalized).first?
  end
end
