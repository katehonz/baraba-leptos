class IsoCountryFactory < Avram::Factory
  def initialize
    alpha2 "BG"
    alpha3 "BGR"
    name "Bulgaria"
    name_bg "България"
    numeric "100"
    is_eu_member true
  end
end
