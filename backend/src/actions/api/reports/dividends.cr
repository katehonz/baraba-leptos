class Api::Reports::Dividends < ApiAction
  include Api::DividendParams

  get "/api/companies/:company_id/reports/dividends" do
    date_from = params.get?("date_from").try(&.to_s) || ""
    date_to = params.get?("date_to").try(&.to_s) || ""
    date_field = params.get?("date_field").try(&.to_s) || "decision_date"
    date_field = "decision_date" unless date_field == "payment_date"
    include_drafts = params.get?("include_drafts").try(&.to_s) == "true"

    dist_map = {} of Int64 => DividendDistribution
    DividendDistributionQuery.new.company_id(company_id).each do |d|
      dist_map[d.id] = d
    end

    owner_map = {} of Int64 => BeneficialOwner
    BeneficialOwnerQuery.new.company_id(company_id).each do |o|
      owner_map[o.id] = o
    end

    rows = [] of NamedTuple(
      id: Int64,
      distribution_id: Int64?,
      year: Int32?,
      decision_date: String?,
      decision_number: String?,
      payment_date: String?,
      owner_name: String?,
      owner_egn: String?,
      ownership_percentage: Float64?,
      gross_amount: Float64,
      tax_rate: Float64,
      tax_amount: Float64,
      net_amount: Float64,
      is_paid: Bool,
      status: String?,
      status_label: String?
    )

    owner_acc = {} of String => NamedTuple(
      owner_name: String,
      owner_egn: String?,
      gross_amount: Float64,
      tax_amount: Float64,
      net_amount: Float64,
      paid_net: Float64,
      count: Int64
    )

    total_gross = 0.0
    total_tax = 0.0
    total_net = 0.0
    total_paid_net = 0.0

    DividendQuery.new.company_id(company_id).each do |d|
      dist = d.dividend_distribution_id.try { |did| dist_map[did]? }
      status = dist.try(&.status)
      next if status == "draft" && !include_drafts

      date_value = if date_field == "payment_date"
                     d.payment_date
                   else
                     d.decision_date
                   end
      next if date_value.nil?

      date_str = format_iso_date(date_value) || ""
      next if !date_from.empty? && date_str < date_from
      next if !date_to.empty? && date_str > date_to

      owner = d.beneficial_owner_id.try { |oid| owner_map[oid]? }
      owner_name = owner.try(&.full_name_bg)
      owner_egn = owner.try(&.egn)
      ownership_percentage = owner.try(&.ownership_percentage)
      decision_date = format_iso_date(d.decision_date) || (dist && format_iso_date(dist.decision_date))
      status_label = status.try { |s| DividendDistribution::STATUSES[s]? } || status

      rows << {
        id:                   d.id,
        distribution_id:      d.dividend_distribution_id,
        year:                 dist.try(&.year),
        decision_date:        decision_date,
        decision_number:      dist.try(&.decision_number),
        payment_date:         format_iso_date(d.payment_date),
        owner_name:           owner_name,
        owner_egn:            owner_egn,
        ownership_percentage: ownership_percentage,
        gross_amount:         d.gross_amount,
        tax_rate:             d.tax_rate,
        tax_amount:           d.tax_amount,
        net_amount:           d.net_amount,
        is_paid:              d.is_paid,
        status:               status,
        status_label:         status_label,
      }

      total_gross += d.gross_amount
      total_tax += d.tax_amount
      total_net += d.net_amount
      total_paid_net += d.is_paid ? d.net_amount : 0.0

      key = "#{d.beneficial_owner_id || 0}:#{owner_name || d.id}"
      if cur = owner_acc[key]?
        owner_acc[key] = {
          owner_name:   cur[:owner_name],
          owner_egn:    cur[:owner_egn],
          gross_amount: cur[:gross_amount] + d.gross_amount,
          tax_amount:   cur[:tax_amount] + d.tax_amount,
          net_amount:   cur[:net_amount] + d.net_amount,
          paid_net:     cur[:paid_net] + (d.is_paid ? d.net_amount : 0.0),
          count:        cur[:count] + 1,
        }
      else
        owner_acc[key] = {
          owner_name:   owner_name || "—",
          owner_egn:    owner_egn,
          gross_amount: d.gross_amount,
          tax_amount:   d.tax_amount,
          net_amount:   d.net_amount,
          paid_net:     d.is_paid ? d.net_amount : 0.0,
          count:        1_i64,
        }
      end
    end

    if date_field == "payment_date"
      rows.sort_by! { |r| {r[:payment_date] || "", r[:owner_name] || "", r[:id]} }
    else
      rows.sort_by! { |r| {r[:decision_date] || "", r[:owner_name] || "", r[:id]} }
    end

    by_owner = owner_acc.values.sort_by { |o| o[:owner_name] }

    json({
      success: true,
      data:    {
        date_from:  date_from.empty? ? nil : date_from,
        date_to:    date_to.empty? ? nil : date_to,
        date_field: date_field,
        rows:       rows,
        totals:     {
          gross_amount: total_gross,
          tax_amount:   total_tax,
          net_amount:   total_net,
          paid_net:     total_paid_net,
          count:        rows.size.to_i64,
        },
        by_owner:   by_owner,
      },
    })
  end
end
