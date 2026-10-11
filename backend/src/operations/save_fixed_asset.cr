class SaveFixedAsset < FixedAsset::SaveOperation
  permit_columns inventory_number, name, description, acquisition_date, put_into_service_date,
    acquisition_cost, residual_value, accounting_book_value, depreciation_method,
    accounting_depreciation_rate, tax_depreciation_rate, status,
    document_number, document_date, company_id, category_id,
    location, conserved_from, conserved_to, disposed_at, disposed_reason,
    sap_accumulated_depreciation, sap_book_value_begin, sap_book_value_end,
    sap_asset_life_months, sap_asset_life_years, sap_valuation_class,
    dap_category, dap_annual_tax_rate, dap_tax_depreciable_value,
    dap_accrued_tax_depreciation, dap_accumulated_depreciation,
    dap_tax_value_begin, dap_tax_value_end, dap_months_depreciation,
    dap_month_suspension, dap_month_write_off_accounting, dap_month_write_off_tax

  before_save do
    if status.value.nil? || status.value == ""
      status.value = "active"
    end
    validate_required inventory_number, name, acquisition_date, acquisition_cost,
      depreciation_method, accounting_depreciation_rate, tax_depreciation_rate
    validate_inclusion_of depreciation_method, in: ["straight_line", "declining_balance"]
    validate_inclusion_of status, in: ["active", "disposed", "fully_depreciated", "conserved"]
    sync_sap_dap_fields
  end

  private def sync_sap_dap_fields
    rate = accounting_depreciation_rate.value
    if rate && rate > 0
      months = (12.0 / (rate / 100.0)).round.to_i
      sap_asset_life_months.value = months
      sap_asset_life_years.value = (months / 12.0).round.to_i
    end

    if tax = tax_depreciation_rate.value
      dap_annual_tax_rate.value = tax
    end

    if cost = acquisition_cost.value
      residual = residual_value.value || 0.0
      sap_acc = sap_accumulated_depreciation.value || 0.0
      tax_acc = dap_accrued_tax_depreciation.value || dap_accumulated_depreciation.value || 0.0

      if (dap_tax_depreciable_value.value || 0.0) <= 0.0
        dap_tax_depreciable_value.value = cost
      end
      sap_book_value_end.value = cost - sap_acc
      accounting_book_value.value = cost - sap_acc
      if (sap_book_value_begin.value || 0.0) <= 0.0
        sap_book_value_begin.value = cost - sap_acc
      end
      dap_tax_value_end.value = (dap_tax_depreciable_value.value || cost) - tax_acc
      if (dap_tax_value_begin.value || 0.0) <= 0.0
        dap_tax_value_begin.value = (dap_tax_depreciable_value.value || cost) - tax_acc
      end
    end

    if cid = category_id.value
      if cat = FixedAssetCategoryQuery.new.id(cid).first?
        if (dap_category.value.nil? || dap_category.value.try(&.empty?)) && cat.cita_category
          dap_category.value = cat.cita_category
        end
      end
    end

    if from = conserved_from.value
      to = conserved_to
      from_s = from.to_s("%m.%Y")
      to_s = to.value.try(&.to_s("%m.%Y")) || ""
      dap_month_suspension.value = [from_s, to_s].reject(&.empty?).join("-") + " консервация"
      if status.value == "active" || status.value.nil?
        status.value = "conserved"
      end
    end

    if at = disposed_at.value
      reason = disposed_reason.value || "отписване"
      dap_month_write_off_accounting.value = "#{at.month} #{reason}"
      dap_month_write_off_tax.value = "#{at.month} #{reason}"
      status.value = "disposed"
    end
  end

  after_save do |asset|
    if at = asset.disposed_at
      already = AssetTransactionQuery.new
        .fixed_asset_id(asset.id)
        .in_date_range(at, at + Time::Span.new(days: 1))
        .first?
      unless already
        reason = asset.disposed_reason || "scrap"
        type_code = case reason
        when "sale" then "20"
        when "scrap" then "50"
        when "gift" then "120"
        else "130"
        end
        book = (asset.accounting_book_value || (asset.acquisition_cost - asset.sap_accumulated_depreciation))
        SaveAssetTransaction.create!(
          asset_transaction_id: "AT-#{asset.id}-DISP-#{at.to_s("%Y%m%d")}",
          asset_transaction_type: type_code,
          description: "Отписване: #{reason}",
          transaction_date: at,
          valuation_type: "SAP",
          acquisition_cost_on_transaction: asset.acquisition_cost,
          book_value_on_transaction: book,
          transaction_amount: book,
          company_id: asset.company_id,
          fixed_asset_id: asset.id
        )
      end
    end
  end
end
