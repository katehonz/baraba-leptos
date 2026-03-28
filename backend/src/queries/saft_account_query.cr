class SaftAccountQuery < SaftAccount::BaseQuery
  def by_code(code_value : String)
    code(code_value)
  end

  def by_code_prefix(prefix : String)
    code.like("#{prefix}%")
  end

  def by_section(section_code_value : String)
    section_code(section_code_value)
  end

  def by_group(group_code_value : String)
    group_code(group_code_value)
  end

  def ordered_by_code
    code.asc_order
  end
end
