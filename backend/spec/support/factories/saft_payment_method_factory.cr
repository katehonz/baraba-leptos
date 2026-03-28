class SaftPaymentMethodFactory < Avram::Factory
  def initialize
    method_code "NU"
    mechanism_code "Nalichni"
    name "Cash payment"
    description "Плащане в брой"
  end
end
