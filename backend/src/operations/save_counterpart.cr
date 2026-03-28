class SaveCounterpart < Counterpart::SaveOperation
  permit_columns name, eik, vat_number, address, city, country_code, contact_person, email, phone, counterpart_type, is_active, company_id

  before_save do
    validate_required name, counterpart_type
    validate_inclusion_of counterpart_type, in: ["customer", "supplier", "both"]
    validate_format_of email, with: /@/, allow_nil: true, message: "must be a valid email"
    generate_saft_id_from_fields if saft_id.value.nil? || saft_id.value.try(&.empty?)
  end

  private def generate_saft_id_from_fields
    eik_val = eik.value
    cc = country_code.value
    vat = vat_number.value

    if eik_val && (cc.nil? || cc == "BG")
      saft_id_type.value = "10"
      saft_id.value = "10#{eik_val}"
    elsif vat && cc
      if Counterpart::EU_COUNTRIES.includes?(cc) && cc != "BG"
        saft_id_type.value = "11"
        saft_id.value = "11#{cc}#{vat}"
      else
        saft_id_type.value = "12"
        saft_id.value = "12#{cc}#{vat || eik_val}"
      end
    elsif eik_val && eik_val.size == 10
      saft_id_type.value = "13"
      saft_id.value = "13#{eik_val}"
    else
      saft_id_type.value = "15"
      saft_id.value = "15#{Time.utc.to_unix}#{Random.rand(9999).to_s.rjust(4, '0')}"
    end
  end
end
