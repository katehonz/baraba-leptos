require "csv"

class Api::SaftAccounts::Load < ApiAction
  include Api::Auth::SkipRequireAuthToken

  post "/api/saft_accounts/load" do
    loaded = load_from_csv
    total = SaftAccountQuery.new.select_count

    json({
      success: true,
      loaded:  loaded,
      total:   total,
      message: "SAF-T accounts loaded successfully",
    })
  end

  private def load_from_csv : Int32
    csv_path = "data/NRA_Nom_Accounts.csv"

    unless File.exists?(csv_path)
      return 0
    end

    accounts_created = 0
    current_section : String? = nil
    current_section_name : String? = nil
    current_group : String? = nil
    current_group_name : String? = nil

    # Pattern for section: "Раздел 1: Сметки за капитал и заеми"
    section_pattern = /Раздел\s+(\d+):\s*(.+)/
    # Pattern for group: "група 10: Капитал" or "Група 10: Капитал"
    group_pattern = /[Гг]рупа\s+(\d+):\s*(.+)/

    CSV.each_row(File.read(csv_path)) do |row|
      next if row.size < 3

      col1 = row[0]?.try(&.strip) || ""
      col2 = row[1]?.try(&.strip) || ""
      col3 = row[2]?.try(&.strip) || ""

      # Check for section header
      if match = section_pattern.match(col3)
        current_section = match[1]
        current_section_name = match[2].strip
        next
      end

      # Check for group header
      if match = group_pattern.match(col3)
        current_group = match[1]
        current_group_name = match[2].strip
        next
      end

      # Check for account entry (col2 contains account code)
      if !col2.empty? && col2.matches?(/^\d+$/)
        code = col2
        # Extract name from col3, removing the code prefix if present
        name = col3.gsub(/^#{code}\s*/, "").strip
        name = col3 if name.empty?

        # Skip if already exists
        existing = SaftAccountQuery.new.by_code(code).first?
        next if existing

        SaveSaftAccount.create!(
          code: code,
          name: name,
          section_code: current_section,
          section_name: current_section_name,
          group_code: current_group,
          group_name: current_group_name
        )
        accounts_created += 1
      end
    end

    accounts_created
  end
end
