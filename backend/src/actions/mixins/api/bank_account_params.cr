# JSON body helpers for bank accounts (flat or nested `bank_account` payload).
# Lucky SaveOperation looks up nested("bank_account") and raises
# MissingNestedParamError when the frontend sends a flat JSON body.
module Api::BankAccountParams
  private def bank_account_payload : JSON::Any
    nested_or_flat(params.from_json, "bank_account")
  end

  private def nested_or_flat(body : JSON::Any, key : String) : JSON::Any
    body[key]? || body
  end

  private def has_key?(data : JSON::Any, key : String) : Bool
    data.as_h?.try(&.has_key?(key)) || false
  end

  private def json_str(val : JSON::Any?) : String?
    return nil if val.nil? || val.raw.nil?
    s = val.as_s?
    return nil if s.nil? || s.blank?
    s.strip
  end

  private def json_i64(val : JSON::Any?) : Int64?
    return nil if val.nil? || val.raw.nil?
    val.as_i64? || val.as_i?.try(&.to_i64) || val.as_s?.try(&.to_i64?)
  end

  private def serialize_bank_account(account : BankAccount)
    {
      id:                account.id,
      name:              account.name,
      iban:              account.iban,
      bic:               account.bic,
      currency:          account.currency,
      bank_name:         account.bank_name,
      gl_account_id:     account.gl_account_id,
      buffer_account_id: account.buffer_account_id,
    }
  end
end
