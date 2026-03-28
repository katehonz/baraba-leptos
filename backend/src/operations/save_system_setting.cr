class SaveSystemSetting < SystemSetting::SaveOperation
  permit_columns key, value, description
end
