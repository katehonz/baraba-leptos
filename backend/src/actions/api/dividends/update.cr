class Api::Dividends::Update < ApiAction
  include Api::DividendParams

  put "/api/companies/:company_id/dividends/:id" do
    dividend = DividendQuery.new
      .id(id)
      .company_id(company_id)
      .first?

    unless dividend
      response.status_code = 404
      return json({success: false, message: "Not found"})
    end

    begin
      data = nested_or_flat(dividend_json_body, "dividend")

      paid = json_bool(data["is_paid"]?)
      paid = paid.nil? ? dividend.is_paid : paid.not_nil!

      payment_date = if paid
                       if s = json_str(data["payment_date"]?)
                         parse_iso_date(s)
                       else
                         dividend.payment_date || Time.utc
                       end
                     else
                       nil
                     end

      notes = data["notes"]? ? json_str(data["notes"]?) : dividend.notes

      updated = SaveDividend.update!(
        dividend,
        is_paid: paid,
        payment_date: payment_date,
        notes: notes
      )

      if updated.dividend_distribution_id
        dist_id = updated.dividend_distribution_id.not_nil!
        all_dividends = DividendQuery.new.dividend_distribution_id(dist_id)
        all_paid = all_dividends.all?(&.is_paid)
        any_paid = all_dividends.any?(&.is_paid)

        distribution = DividendDistributionQuery.new.id(dist_id).first?
        if distribution
          new_status = if all_paid
                         "paid"
                       elsif any_paid
                         "partially_paid"
                       else
                         distribution.status == "approved" ? "approved" : distribution.status
                       end

          if new_status != distribution.status
            SaveDividendDistribution.update!(distribution, status: new_status)
          end
        end
      end

      json({
        success: true,
        data:    serialize_dividend(updated),
      })
    rescue ex : Time::Format::Error
      response.status_code = 422
      json({
        success: false,
        message: "Невалиден формат на дата (очакван YYYY-MM-DD)",
        errors:  [{field: "payment_date", messages: [ex.message.to_s]}],
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
