class SaftInvoiceTypeFactory < Avram::Factory
  def initialize
    code "FT"
    name "Factura"
    description "Фактура за доставени стоки и оказани услуги на територията на страната от регистрирано лице по ЗДДС"
  end
end
