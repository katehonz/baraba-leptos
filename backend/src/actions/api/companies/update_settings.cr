class Api::Companies::UpdateSettings < ApiAction
  put "/api/companies/:id/settings" do
    company = CompanyQuery.new.id(id).first?

    unless company
      response.status_code = 404
      return json({success: false, message: "Company not found"})
    end

    # Parse JSON body
    body_str = request.body.try(&.gets_to_end) || "{}"
    json_body = JSON.parse(body_str)

    # Get new settings from request
    if new_settings = json_body["settings"]?.try(&.as_h?)
      # Merge with existing settings
      existing_settings = company.parsed_settings.as_h? || {} of String => JSON::Any

      new_settings.each do |key, value|
        existing_settings[key] = value
      end

      # Update company settings
      CompanyQuery.new.id(id).update(settings: existing_settings.to_json)

      # Reload and return
      updated_company = CompanyQuery.new.id(id).first?
      if updated_company
        json({
          success: true,
          data: JsonbSerializer.serialize_company(updated_company),
        })
      else
        response.status_code = 404
        json({success: false, message: "Company not found"})
      end
    else
      response.status_code = 400
      json({success: false, message: "Missing settings in request body"})
    end
  end
end
