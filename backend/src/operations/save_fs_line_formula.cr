class SaveFsLineFormula < FsLineFormula::SaveOperation
  permit_columns statement, line_code, formula, company_id

  before_save do
    validate_required statement, line_code
    if st = statement.value
      unless FsCatalog::STATEMENTS.includes?(st)
        statement.add_error "непознат отчет"
      end
    end
    if code = line_code.value
      if st = statement.value
        unless FsCatalog.has_code?(st, code)
          line_code.add_error "непозната статия"
        end
      end
    end
    if f = formula.value
      if err = FsFormula.validate(f, statement.value)
        formula.add_error err
      end
    end
  end
end
