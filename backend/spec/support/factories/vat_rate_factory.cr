class VatRateFactory < Avram::Factory
  def initialize
    name "Standard"
    percentage 20.0
    code "S"
    saft_tax_type "IVA"
    saft_tax_code "NOR"
    effective_from Time.utc(2007, 1, 1)
    is_active true
  end
end
