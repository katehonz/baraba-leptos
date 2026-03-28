# SAF-T IBAN Format reference table
# IBAN formats by country (ISO 13616)
class SaftIbanFormat < BaseModel
  table do
    column country : String        # Country name
    column length : Int32          # Total IBAN length
    column bban_format : String?   # BBAN format description
    column iban_fields : String?   # IBAN field structure
    column comments : String?      # Additional notes
  end

  def display_name : String
    "#{country} (#{length} chars)"
  end

  # Validate IBAN format for a given country
  def self.validate_iban(iban : String, country_code : String) : Bool
    # Remove spaces and convert to uppercase
    clean_iban = iban.gsub(/\s/, "").upcase

    # Get format for country
    format = SaftIbanFormatQuery.new.country(country_code).first?
    return false unless format

    # Check length
    return false if clean_iban.size != format.length

    # Validate checksum (ISO 7064 Mod 97-10)
    rearranged = clean_iban[4..] + clean_iban[0..3]
    numeric_string = rearranged.chars.map do |c|
      c.ascii_number? ? c.to_s : (c.ord - 55).to_s
    end.join

    numeric_string.to_big_i % 97 == 1
  end
end
