require "../../../services/saft_exporter"

class Api::Saft::Export < ApiAction
  include Api::Auth::SkipRequireAuthToken

  param year : Int32
  param month : Int32
  param company_id : Int64
  param report_type : String = "monthly"

  get "/api/saft/export" do
    user = current_user?
    return json({error: "Unauthorized"}, status: 401) unless user

    # Verify user has access to company
    company = CompanyQuery.new.find(company_id)

    # Check authorization (user must own company or have role)
    unless company.owner_id == user.id || user_has_company_role?(user, company)
      return json({error: "Unauthorized"}, status: 403)
    end

    # Determine period
    period_start = Time.utc(year, month, 1)
    period_end = period_start.at_end_of_month

    # Determine report type
    saft_report_type = case report_type.downcase
    when "monthly"   then SaftExporter::ReportType::Monthly
    when "ondemand"  then SaftExporter::ReportType::OnDemand
    when "annual"    then SaftExporter::ReportType::Annual
    else             SaftExporter::ReportType::Monthly
    end

    # For annual reports, use full year
    if saft_report_type == SaftExporter::ReportType::Annual
      period_start = Time.utc(year, 1, 1)
      period_end = Time.utc(year, 12, 31, 23, 59, 59)
    end

    # Generate SAF-T XML
    exporter = SaftExporter.new(company, period_start, period_end, saft_report_type)
    xml_content = exporter.generate

    # Generate filename
    filename = "SAFT_#{company.eik}_#{year}#{month.to_s.rjust(2, '0')}_#{report_type}.xml"

    # Return as file download
    response.content_type = "application/xml"
    response.headers["Content-Disposition"] = "attachment; filename=\"#{filename}\""
    plain_text xml_content
  end

  private def user_has_company_role?(user : User, company : Company) : Bool
    !UserCompanyRoleQuery.new.user_id(user.id).company_id(company.id).first?.nil?
  end
end
