class SaftTaxTypeFactory < Avram::Factory
  def initialize
    code "IVA"
    name "DDS"
    description "Данък върху добавената стойност"
  end
end
