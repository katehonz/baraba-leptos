class SaveCounterpart < Counterpart::SaveOperation
  permit_columns name, eik, vat_number, address, city, country_code, contact_person, email, phone, counterpart_type, is_active, company_id

  before_save do
    normalize_identifiers
    validate_required name, counterpart_type
    validate_inclusion_of counterpart_type, in: ["customer", "supplier", "both"]
    validate_format_of email, with: /@/, allow_nil: true, message: "must be a valid email"
    validate_eik_unique
    generate_saft_id_from_fields if saft_id.value.nil? || saft_id.value.try(&.empty?)
  end

  private def normalize_identifiers
    if raw_vat = vat_number.value
      cleaned_vat = raw_vat.strip.upcase.gsub(/[\s.\-]/, "")
      vat_number.value = cleaned_vat.empty? ? nil : cleaned_vat
    end

    if raw_eik = eik.value
      eik.value = Counterpart.normalize_eik(raw_eik)
    end

    if eik.value.nil? || eik.value.try(&.empty?)
      if vat = vat_number.value
        extracted = Counterpart.normalize_eik(vat)
        eik.value = extracted unless extracted.nil?
      end
    end
  end

  private def validate_eik_unique
    eik_val = eik.value
    return if eik_val.nil? || eik_val.empty?
    return if Counterpart.generic_eik?(eik_val)

    cid = company_id.value
    return unless cid

    query = CounterpartQuery.new.company_id(cid).eik(eik_val)
    if rec = record
      query = query.id.not.eq(rec.id)
    end

    if existing = query.first?
      eik.add_error("Контрагент с ЕИК #{eik_val} вече съществува: #{existing.name}")
    end
  end

  private def generate_saft_id_from_fields
    eik_val = eik.value
    cc = country_code.value
    vat = vat_number.value

    # Placeholder EIK (9/13 nines) can repeat, so SAF-T ID must stay unique.
    if Counterpart.generic_eik?(eik_val)
      saft_id_type.value = "15"
      saft_id.value = unique_internal_saft_id
    elsif eik_val && (cc.nil? || cc == "BG")
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
      saft_id.value = unique_internal_saft_id
    end
  end

  private def unique_internal_saft_id : String
    "15#{Time.utc.to_unix_ms}#{Random.rand(1_000_000).to_s.rjust(6, '0')}"
  end
end
