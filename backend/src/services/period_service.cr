# Service for accounting period validation
module PeriodService
  extend self

  # Check if a period is open for a given date
  def period_open?(company_id : Int64, date : Time) : Bool
    period = AccountingPeriodQuery.new
      .company_id(company_id)
      .year(date.year)
      .month(date.month)
      .first?

    # If no period record exists, it's considered open
    period.nil? || period.status == "open"
  end

  # Raise an error if the period is closed
  def validate_period_open!(company_id : Int64, date : Time)
    unless period_open?(company_id, date)
      raise PeriodClosedError.new(date.year, date.month)
    end
  end

  # Get period status for a date
  def get_period_status(company_id : Int64, date : Time) : String
    period = AccountingPeriodQuery.new
      .company_id(company_id)
      .year(date.year)
      .month(date.month)
      .first?

    period.try(&.status) || "open"
  end

  # Close a period
  def close_period(company_id : Int64, year : Int32, month : Int32, user_id : Int64, notes : String? = nil) : AccountingPeriod
    period = AccountingPeriodQuery.new
      .company_id(company_id)
      .year(year)
      .month(month)
      .first?

    if period
      SaveAccountingPeriod.update!(period,
        status: "closed",
        closed_at: Time.utc,
        closed_by_id: user_id,
        notes: notes || period.notes
      )
      AccountingPeriodQuery.new.id(period.id).first
    else
      SaveAccountingPeriod.create!(
        company_id: company_id,
        year: year,
        month: month,
        status: "closed",
        closed_at: Time.utc,
        closed_by_id: user_id,
        notes: notes
      )
    end
  end

  # Reopen a period
  def reopen_period(company_id : Int64, year : Int32, month : Int32) : AccountingPeriod?
    period = AccountingPeriodQuery.new
      .company_id(company_id)
      .year(year)
      .month(month)
      .first?

    return nil unless period

    SaveAccountingPeriod.update!(period,
      status: "open",
      closed_at: nil,
      closed_by_id: nil
    )

    AccountingPeriodQuery.new.id(period.id).first
  end
end

# Custom error for closed period
class PeriodClosedError < Exception
  getter year : Int32
  getter month : Int32

  def initialize(@year, @month)
    months = ["", "Януари", "Февруари", "Март", "Април", "Май", "Юни",
              "Юли", "Август", "Септември", "Октомври", "Ноември", "Декември"]
    super("Периодът #{months[@month]} #{@year} е затворен. Не може да се редактират данни в затворен период.")
  end
end
