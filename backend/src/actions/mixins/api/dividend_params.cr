# Общи помощници за JSON тяло на дивиденти (плосък или вложен payload)
module Api::DividendParams
  private def dividend_json_body : JSON::Any
    body_str = request.body.try(&.gets_to_end) || "{}"
    JSON.parse(body_str)
  rescue
    JSON::Any.new({} of String => JSON::Any)
  end

  private def nested_or_flat(body : JSON::Any, key : String) : JSON::Any
    body[key]? || body
  end

  private def json_i32(val : JSON::Any?) : Int32?
    return nil if val.nil?
    val.as_i? || val.as_i64?.try(&.to_i32) || val.as_s?.try(&.to_i32?)
  end

  private def json_i64(val : JSON::Any?) : Int64?
    return nil if val.nil?
    val.as_i64? || val.as_i?.try(&.to_i64) || val.as_s?.try(&.to_i64?)
  end

  private def json_f64(val : JSON::Any?) : Float64?
    return nil if val.nil?
    val.as_f? || val.as_i64?.try(&.to_f) || val.as_i?.try(&.to_f) || val.as_s?.try(&.gsub(",", ".").to_f?)
  end

  private def json_bool(val : JSON::Any?) : Bool?
    return nil if val.nil?
    case val.raw
    when Bool
      val.as_bool
    when String
      s = val.as_s.downcase
      return true if s == "true" || s == "1"
      return false if s == "false" || s == "0"
      nil
    else
      nil
    end
  end

  private def json_str(val : JSON::Any?) : String?
    return nil if val.nil?
    s = val.as_s?
    return nil if s.nil? || s.blank?
    s
  end

  private def parse_iso_date(value : String) : Time
    slice = value.size >= 10 ? value[0, 10] : value
    Time.parse(slice, "%Y-%m-%d", Time::Location::UTC)
  end

  private def format_iso_date(value : Time?) : String?
    value.try(&.to_s("%Y-%m-%d"))
  end

  private def serialize_dividend(d : Dividend)
    owner = d.beneficial_owner_id ? BeneficialOwnerQuery.new.id(d.beneficial_owner_id.not_nil!).first? : nil
    {
      id:                   d.id,
      beneficial_owner_id:  d.beneficial_owner_id,
      owner_name:           owner.try(&.full_name_bg),
      owner_egn:            owner.try(&.egn),
      ownership_percentage: owner.try(&.ownership_percentage),
      gross_amount:         d.gross_amount,
      tax_rate:             d.tax_rate,
      tax_amount:           d.tax_amount,
      net_amount:           d.net_amount,
      is_paid:              d.is_paid,
      payment_date:         format_iso_date(d.payment_date),
    }
  end

  private def serialize_distribution(distribution : DividendDistribution, dividends : DividendQuery)
    {
      id:              distribution.id,
      year:            distribution.year,
      total_amount:    distribution.total_amount,
      decision_date:   format_iso_date(distribution.decision_date),
      decision_number: distribution.decision_number,
      status:          distribution.status,
      status_label:    DividendDistribution::STATUSES[distribution.status]?,
      notes:           distribution.notes,
      created_at:      format_iso_date(distribution.created_at),
      dividends:       dividends.map { |d| serialize_dividend(d) },
    }
  end
end
