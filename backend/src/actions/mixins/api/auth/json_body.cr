module Api::Auth::JsonBody
  private def json_body : Hash(String, JSON::Any)
    raw = request.body.try(&.gets_to_end) || "{}"
    raw = "{}" if raw.blank?
    JSON.parse(raw).as_h
  rescue
    {} of String => JSON::Any
  end

  private def json_field(body : Hash(String, JSON::Any), key : String) : String
    body[key]?.try(&.as_s?) || ""
  end
end
