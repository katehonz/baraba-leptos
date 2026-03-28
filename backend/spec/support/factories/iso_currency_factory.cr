class IsoCurrencyFactory < Avram::Factory
  def initialize
    code "BGN"
    name "Bulgarian Lev"
    name_bg "Български лев"
    numeric "975"
    symbol "лв"
    decimal_places 2
  end
end
