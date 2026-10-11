class Api::DividendDistributions::Create < ApiAction
  include Api::DividendParams

  post "/api/companies/:company_id/dividend_distributions" do
    begin
      data = nested_or_flat(dividend_json_body, "dividend_distribution")

      year = json_i32(data["year"]?)
      total_amount = json_f64(data["total_amount"]?)
      decision_date_str = json_str(data["decision_date"]?)
      company = CompanyQuery.new.id(company_id).first?
      default_tax_rate = company.try(&.dividend_tax_rate) || DividendDistribution::TAX_RATE
      tax_rate = json_f64(data["tax_rate"]?) || default_tax_rate

      if year.nil?
        response.status_code = 422
        return json({
          success: false,
          message: "Финансовата година е задължителна",
          errors:  [{field: "year", messages: ["е задължителна"]}],
        })
      end

      if total_amount.nil? || total_amount.not_nil! <= 0
        response.status_code = 422
        return json({
          success: false,
          message: "Сумата за разпределяне трябва да е по-голяма от 0",
          errors:  [{field: "total_amount", messages: ["трябва да е по-голяма от 0"]}],
        })
      end

      if decision_date_str.nil?
        response.status_code = 422
        return json({
          success: false,
          message: "Датата на решение е задължителна",
          errors:  [{field: "decision_date", messages: ["е задължителна"]}],
        })
      end

      decision_date = parse_iso_date(decision_date_str.not_nil!)
      decision_number = json_str(data["decision_number"]?)
      notes = json_str(data["notes"]?)

      beneficial_owners = BeneficialOwnerQuery.new
        .company_id(company_id)
        .is_active(true)
        .to_a
        .select { |bo| bo.ownership_percentage && bo.ownership_percentage.not_nil! > 0 }

      if beneficial_owners.empty?
        response.status_code = 422
        return json({
          success: false,
          message: "Няма действителни собственици с процент на собственост. Добавете ги в Настройки → SAFt.",
          errors:  [{field: "base", messages: ["Няма действителни собственици с процент на собственост. Добавете ги в Настройки → SAFt."]}],
        })
      end

      distribution = SaveDividendDistribution.create!(
        company_id: company_id.to_i64,
        year: year.not_nil!,
        total_amount: total_amount.not_nil!,
        decision_date: decision_date,
        decision_number: decision_number,
        status: "draft",
        notes: notes
      )

      calculated = distribution.calculate_dividends(beneficial_owners, tax_rate)
      calculated.each do |calc|
        SaveDividend.create!(
          company_id: company_id.to_i64,
          dividend_distribution_id: distribution.id,
          beneficial_owner_id: calc[:beneficial_owner_id],
          gross_amount: calc[:gross_amount],
          tax_rate: calc[:tax_rate],
          tax_amount: calc[:tax_amount],
          net_amount: calc[:net_amount],
          decision_date: distribution.decision_date,
          is_paid: false
        )
      end

      dividends = DividendQuery.new.dividend_distribution_id(distribution.id)
      json({
        success: true,
        data:    serialize_distribution(distribution, dividends),
      })
    rescue ex : Time::Format::Error
      response.status_code = 422
      json({
        success: false,
        message: "Невалиден формат на дата (очакван YYYY-MM-DD)",
        errors:  [{field: "decision_date", messages: [ex.message.to_s]}],
      })
    rescue ex : Avram::InvalidOperationError
      response.status_code = 422
      json({
        success: false,
        message: "Грешка при валидация: #{ex.message}",
        errors:  [{field: "base", messages: [ex.message.to_s]}],
      })
    rescue ex
      Log.error { "Dividend distribution create error: #{ex.message}" }
      response.status_code = 500
      json({
        success: false,
        message: "Грешка при създаване: #{ex.message}",
      })
    end
  end
end
