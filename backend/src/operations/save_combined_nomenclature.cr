class SaveCombinedNomenclature < CombinedNomenclature::SaveOperation
  permit_columns cn_code, description, primary_unit, supplementary_unit,
    level, year, parent_code, is_header

  before_save do
    validate_required cn_code, description, level, year
  end
end
