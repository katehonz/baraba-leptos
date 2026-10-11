# Monthly SAP/DAP depreciation as in EJ FIXA and NSS 4 / ЗКПО.
#
# Depreciation starts the month after put-into-service.
# Conservation (чл. 59 ЗКПО / FIXA): skip from the month after conserved_from
# through the month of conserved_to. Disposal stops from the month after disposed_at.
module DepreciationEngine
  struct Params
    getter cost : Float64
    getter residual : Float64
    getter rate : Float64
    getter service_date : Time
    getter opening_accumulated : Float64
    getter opening_as_of : Time?
    getter conserved_from : Time?
    getter conserved_to : Time?
    getter disposed_at : Time?

    def initialize(
      @cost : Float64,
      @residual : Float64,
      @rate : Float64,
      @service_date : Time,
      @opening_accumulated : Float64 = 0.0,
      @opening_as_of : Time? = nil,
      @conserved_from : Time? = nil,
      @conserved_to : Time? = nil,
      @disposed_at : Time? = nil
    )
    end
  end

  struct MonthRow
    getter year : Int32
    getter month : Int32
    getter cost : Float64
    getter useful_months : Int32
    getter monthly_rate : Float64
    getter amount : Float64
    getter accumulated : Float64
    getter remaining : Float64
    getter conserved : Bool
    getter note : String

    def initialize(
      @year : Int32,
      @month : Int32,
      @cost : Float64,
      @useful_months : Int32,
      @monthly_rate : Float64,
      @amount : Float64,
      @accumulated : Float64,
      @remaining : Float64,
      @conserved : Bool,
      @note : String
    )
    end
  end

  def self.useful_life_months(rate : Float64) : Int32
    return 0 if rate <= 0.0
    (12.0 / (rate / 100.0)).round.to_i
  end

  def self.monthly_amount(cost : Float64, residual : Float64, rate : Float64) : Float64
    depreciable = cost - residual
    return 0.0 if depreciable <= 0.0 || rate <= 0.0
    ((cost * (rate / 100.0)) / 12.0).round(2)
  end

  def self.start_year_month(service : Time) : {Int32, Int32}
    year = service.year
    month = service.month + 1
    if month > 12
      {year + 1, 1}
    else
      {year, month}
    end
  end

  # Skip from the month AFTER `from` through the month of `to` (or ongoing).
  def self.conserved?(from : Time?, to : Time?, year : Int32, month : Int32) : Bool
    return false unless from
    skip_year, skip_month = start_year_month(from)
    current = year * 12 + month
    return false if current < (skip_year * 12 + skip_month)
    if to
      current <= (to.year * 12 + to.month)
    else
      true
    end
  end

  def self.disposed?(at : Time?, year : Int32, month : Int32) : Bool
    return false unless at
    stop_year, stop_month = start_year_month(at)
    (year * 12 + month) >= (stop_year * 12 + stop_month)
  end

  def self.for_asset(asset : FixedAsset, sap : Bool) : Params
    residual = asset.residual_value || 0.0
    service = asset.put_into_service_date || asset.acquisition_date
    if sap
      Params.new(
        cost: asset.acquisition_cost,
        residual: residual,
        rate: asset.accounting_depreciation_rate,
        service_date: service,
        conserved_from: asset.conserved_from,
        conserved_to: asset.conserved_to,
        disposed_at: asset.disposed_at
      )
    else
      cost = asset.dap_tax_depreciable_value > 0 ? asset.dap_tax_depreciable_value : asset.acquisition_cost
      rate = asset.dap_annual_tax_rate > 0 ? asset.dap_annual_tax_rate : asset.tax_depreciation_rate
      Params.new(
        cost: cost,
        residual: 0.0,
        rate: rate,
        service_date: service,
        conserved_from: asset.conserved_from,
        conserved_to: asset.conserved_to,
        disposed_at: asset.disposed_at
      )
    end
  end

  # Months from the first depreciation month through year/month inclusive.
  def self.schedule(params : Params, through_year : Int32, through_month : Int32) : Array(MonthRow)
    rows = [] of MonthRow
    depreciable = params.cost - params.residual
    return rows if depreciable <= 0.0 || params.rate <= 0.0

    base = monthly_amount(params.cost, params.residual, params.rate)
    life = useful_life_months(params.rate)
    monthly_rate = (params.rate / 100.0 / 12.0)
    sy, sm = start_year_month(params.service_date)

    if as_of = params.opening_as_of
      oy, om = start_year_month(as_of)
      if (oy * 12 + om) > (sy * 12 + sm)
        sy, sm = oy, om
      end
    end

    accumulated = params.opening_accumulated
    accumulated = depreciable if accumulated > depreciable
    year = sy
    month = sm

    while (year * 12 + month) <= (through_year * 12 + through_month)
      conserved = conserved?(params.conserved_from, params.conserved_to, year, month)
      disposed = disposed?(params.disposed_at, year, month)
      remaining = (depreciable - accumulated).round(2)
      remaining = 0.0 if remaining < 0.0

      amount = 0.0
      note = ""
      if disposed
        note = "отписване"
      elsif conserved
        note = "консервация"
      elsif remaining <= 0.0
        note = "напълно амортизиран"
      else
        amount = base
        amount = remaining if amount > remaining
        amount = 0.0 if amount < 0.0
      end

      accumulated = (accumulated + amount).round(2)
      remaining = (depreciable - accumulated).round(2)
      remaining = 0.0 if remaining < 0.0

      rows << MonthRow.new(
        year: year,
        month: month,
        cost: params.cost,
        useful_months: life,
        monthly_rate: monthly_rate,
        amount: amount,
        accumulated: accumulated,
        remaining: remaining,
        conserved: conserved,
        note: note
      )

      month += 1
      if month > 12
        month = 1
        year += 1
      end
    end

    rows
  end

  def self.amount_for_month(params : Params, year : Int32, month : Int32) : Float64
    row = schedule(params, year, month).last?
    row ? row.amount : 0.0
  end

  def self.year_months(params : Params, year : Int32) : Array(MonthRow)
    schedule(params, year, 12).select { |r| r.year == year }
  end
end
