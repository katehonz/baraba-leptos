# Resolves tax base / VAT / gross for Bulgarian invoices.
#
# JournalEntry.total_amount is stored inconsistently:
#   * AI scan historically saved the NET tax base (subtotal)
#   * manual journal UI and Controlisy save GROSS (sum of debit lines)
#
# For 20% VAT the tax base is gross / 1.20 (user-facing formula from the
# invoice total). 9% uses gross / 1.09. Prefer journal lines when present.
module VatAmounts
  STANDARD_RATE = 0.20
  REDUCED_RATE  = 0.09
  TOLERANCE     = 0.05

  def self.tax_base_from_gross(gross : Float64, rate : Float64 = STANDARD_RATE) : Float64
    return 0.0 if gross == 0.0
    (gross / (1.0 + rate)).round(2)
  end

  def self.vat_from_base(base : Float64, rate : Float64 = STANDARD_RATE) : Float64
    (base * rate).round(2)
  end

  def self.vat_from_lines(lines_str : String?) : Float64
    JournalLines.parse(lines_str).sum(0.0) do |line|
      code = JournalLines.account_code(line) || ""
      if vat_account?(code)
        debit = JournalLines.debit(line)
        credit = JournalLines.credit(line)
        debit > 0 ? debit : credit
      else
        0.0
      end
    end.round(2)
  end

  def self.vat_account?(code : String) : Bool
    code.starts_with?("453")
  end

  def self.matches_rate?(base : Float64, vat : Float64, rate : Float64, tolerance : Float64 = TOLERANCE) : Bool
    (vat - vat_from_base(base, rate)).abs <= tolerance
  end

  def self.looks_like_net?(stored : Float64, vat : Float64) : Bool
    return false if vat.abs <= TOLERANCE || stored.abs <= TOLERANCE
    matches_rate?(stored, vat, STANDARD_RATE) || matches_rate?(stored, vat, REDUCED_RATE)
  end

  def self.looks_like_gross?(stored : Float64, vat : Float64) : Bool
    return false if vat.abs <= TOLERANCE || stored.abs <= TOLERANCE
    [STANDARD_RATE, REDUCED_RATE].any? do |rate|
      base = tax_base_from_gross(stored, rate)
      implied_vat = (stored - base).round(2)
      (vat - implied_vat).abs <= TOLERANCE
    end
  end

  def self.from_journal_entry(entry : JournalEntry) : NamedTuple(base: Float64, vat: Float64, gross: Float64)
    stored_vat = (entry.total_vat_amount || 0.0).round(2)
    line_vat = vat_from_lines(entry.lines)
    vat = stored_vat.abs > TOLERANCE ? stored_vat : line_vat

    stored = (entry.total_amount || 0.0).round(2)
    line_debit = JournalLines.total_debit(entry.lines).round(2)

    gross = resolve_gross(stored, vat, line_debit)
    base = resolve_base(gross, vat)

    {base: base, vat: vat.round(2), gross: gross}
  end

  def self.check_amounts(base : Float64, vat : Float64, gross : Float64? = nil) : NamedTuple(ok: Bool, messages: Array(String), expected_base: Float64, expected_vat: Float64)
    total = (gross || (base + vat)).round(2)
    expected_base = tax_base_from_gross(total, STANDARD_RATE)
    expected_vat = (total - expected_base).round(2)
    messages = [] of String

    if (base + vat - total).abs > TOLERANCE
      messages << "Основа (#{fmt(base)}) + ДДС (#{fmt(vat)}) ≠ обща сума (#{fmt(total)})"
    end

    if vat.abs > TOLERANCE
      unless matches_rate?(base, vat, STANDARD_RATE) || matches_rate?(base, vat, REDUCED_RATE)
        messages << "ДДС не е 20% или 9% от основата. За 20%: основа = обща сума / 1,20 = #{fmt(expected_base)}, ДДС = #{fmt(expected_vat)}"
      end
    end

    {ok: messages.empty?, messages: messages, expected_base: expected_base, expected_vat: expected_vat}
  end

  private def self.resolve_gross(stored : Float64, vat : Float64, line_debit : Float64) : Float64
    if vat.abs <= TOLERANCE
      return line_debit > 0 ? line_debit : stored
    end

    # AI scan stored NET as total_amount; lines may omit the VAT debit.
    if looks_like_net?(stored, vat) && (line_debit <= 0 || (line_debit - stored).abs <= TOLERANCE)
      return (stored + vat).round(2)
    end

    return line_debit if line_debit > 0
    return stored if looks_like_gross?(stored, vat)
    return (stored + vat).round(2) if looks_like_net?(stored, vat)

    stored
  end

  private def self.resolve_base(gross : Float64, vat : Float64) : Float64
    return gross.round(2) if vat.abs <= TOLERANCE

    standard_base = tax_base_from_gross(gross, STANDARD_RATE)
    return standard_base if matches_rate?(standard_base, vat, STANDARD_RATE)

    reduced_base = tax_base_from_gross(gross, REDUCED_RATE)
    return reduced_base if matches_rate?(reduced_base, vat, REDUCED_RATE)

    (gross - vat).round(2)
  end

  private def self.fmt(amount : Float64) : String
    sprintf("%.2f", amount)
  end
end
