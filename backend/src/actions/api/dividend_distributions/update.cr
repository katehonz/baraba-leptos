class Api::DividendDistributions::Update < ApiAction
  include Api::DividendParams

  put "/api/companies/:company_id/dividend_distributions/:id" do
    distribution = DividendDistributionQuery.new
      .id(id)
      .company_id(company_id)
      .first?

    unless distribution
      response.status_code = 404
      return json({success: false, message: "Not found"})
    end

    begin
      data = nested_or_flat(dividend_json_body, "dividend_distribution")
      existing = DividendQuery.new.dividend_distribution_id(distribution.id).to_a
      any_paid = existing.any?(&.is_paid)

      status_only = json_str(data["status"]?).present? &&
        json_f64(data["total_amount"]?).nil? &&
        json_i32(data["year"]?).nil? &&
        json_str(data["decision_date"]?).nil? &&
        json_f64(data["tax_rate"]?).nil?

      new_status = json_str(data["status"]?) || distribution.status
      unless DividendDistribution::STATUSES.has_key?(new_status)
        response.status_code = 422
        return json({
          success: false,
          message: "Невалиден статус",
          errors:  [{field: "status", messages: ["невалиден статус"]}],
        })
      end

      new_year = json_i32(data["year"]?) || distribution.year
      new_total = json_f64(data["total_amount"]?) || distribution.total_amount
      new_decision_date = if s = json_str(data["decision_date"]?)
                            parse_iso_date(s)
                          else
                            distribution.decision_date
                          end
      new_decision_number = data["decision_number"]? ? json_str(data["decision_number"]?) : distribution.decision_number
      new_notes = data["notes"]? ? json_str(data["notes"]?) : distribution.notes

      company = CompanyQuery.new.id(company_id).first?
      default_tax = existing.first?.try(&.tax_rate) || company.try(&.dividend_tax_rate) || DividendDistribution::TAX_RATE
      tax_rate = json_f64(data["tax_rate"]?) || default_tax

      amounts_changed = (new_total - distribution.total_amount).abs > 0.0001 ||
        (tax_rate - default_tax).abs > 0.0001

      if amounts_changed && any_paid
        response.status_code = 422
        return json({
          success: false,
          message: "Не може да се променя брутната сума или данъкът, когато има изплатени дивиденти. Първо отменете плащанията.",
          errors:  [{field: "total_amount", messages: ["има изплатени дивиденти"]}],
        })
      end

      if new_total <= 0
        response.status_code = 422
        return json({
          success: false,
          message: "Сумата за разпределяне трябва да е по-голяма от 0",
          errors:  [{field: "total_amount", messages: ["трябва да е по-голяма от 0"]}],
        })
      end

      reverting_to_draft = new_status == "draft" && distribution.status != "draft"

      if reverting_to_draft
        existing.each do |row|
          next unless row.is_paid || row.payment_date
          SaveDividend.update!(row, is_paid: false, payment_date: nil)
        end
        existing = DividendQuery.new.dividend_distribution_id(distribution.id).to_a
        any_paid = false
      end

      updated = SaveDividendDistribution.update!(
        distribution,
        year: new_year,
        total_amount: new_total,
        decision_date: new_decision_date,
        decision_number: new_decision_number,
        status: new_status,
        notes: new_notes
      )

      unless status_only
        beneficial_owners = BeneficialOwnerQuery.new
          .company_id(company_id)
          .is_active(true)
          .to_a
          .select { |bo| bo.ownership_percentage && bo.ownership_percentage.not_nil! > 0 }

        calculated = updated.calculate_dividends(beneficial_owners, tax_rate)
        keep_ids = [] of Int64

        calculated.each do |calc|
          if row = existing.find { |d| d.beneficial_owner_id == calc[:beneficial_owner_id] }
            SaveDividend.update!(
              row,
              gross_amount: calc[:gross_amount],
              tax_rate: calc[:tax_rate],
              tax_amount: calc[:tax_amount],
              net_amount: calc[:net_amount],
              decision_date: updated.decision_date
            )
            keep_ids << row.id
          else
            created = SaveDividend.create!(
              company_id: company_id.to_i64,
              dividend_distribution_id: updated.id,
              beneficial_owner_id: calc[:beneficial_owner_id],
              gross_amount: calc[:gross_amount],
              tax_rate: calc[:tax_rate],
              tax_amount: calc[:tax_amount],
              net_amount: calc[:net_amount],
              decision_date: updated.decision_date,
              is_paid: false
            )
            keep_ids << created.id
          end
        end

        unless any_paid
          existing.each do |row|
            next if keep_ids.includes?(row.id)
            DeleteDividend.delete!(row)
          end
        end
      end

      dividends = DividendQuery.new.dividend_distribution_id(updated.id)
      json({
        success: true,
        data:    serialize_distribution(updated, dividends),
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
    end
  end
end
