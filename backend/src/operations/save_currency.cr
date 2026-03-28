class SaveCurrency < Currency::SaveOperation
  permit_columns code, name, name_bg, symbol, decimal_places, is_active, is_base_currency

  before_save do
    validate_required code, name
    validate_size_of code, is: 3
  end
end
