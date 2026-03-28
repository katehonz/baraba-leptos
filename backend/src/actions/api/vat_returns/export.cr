# Export VAT return files (DEKLAR.TXT, POKUPKI.TXT, PRODAJBI.TXT)
# Format: Fixed-width text files per NAP (НАП) specifications
# Encoding: Windows-1251 (converted from UTF-8)
class Api::VatReturns::Export < ApiAction
  include Api::Auth::SkipRequireAuthToken

  # Line terminator for NAP files (CRLF)
  CRLF = "\r\n"

  get "/api/companies/:company_id/vat_returns/:vat_return_id/export/:export_type" do
    vat_return = VatReturnQuery.new
      .company_id(company_id)
      .id(vat_return_id)
      .first?

    unless vat_return
      return json({success: false, error: "VAT return not found"})
    end

    company = CompanyQuery.new.id(company_id).first

    case export_type
    when "deklar"
      content = generate_deklar(company, vat_return)
      filename = "DEKLAR.TXT"
    when "pokupki"
      content = generate_pokupki(company, vat_return)
      filename = "POKUPKI.TXT"
    when "prodajbi"
      content = generate_prodajbi(company, vat_return)
      filename = "PRODAGBI.TXT"
    else
      return json({success: false, error: "Unknown export type: #{export_type}"})
    end

    # Convert to Windows-1251 encoding using iconv
    encoded_content = convert_to_win1251(content)

    response.content_type = "text/plain; charset=windows-1251"
    response.headers["Content-Disposition"] = "attachment; filename=\"#{filename}\""

    plain_text encoded_content
  end

  # Convert UTF-8 string to Windows-1251
  private def convert_to_win1251(content : String) : String
    # Write content to temp file, convert with gnu-iconv, read result
    temp_in = "/tmp/vat_export_in_#{Random.new.hex(8)}.txt"
    temp_out = "/tmp/vat_export_out_#{Random.new.hex(8)}.txt"

    begin
      # Write UTF-8 content to temp file
      File.write(temp_in, content)

      # Convert using gnu-iconv (Alpine's musl iconv doesn't support Windows-1251)
      status = Process.run(
        "sh",
        ["-c", "gnu-iconv -f UTF-8 -t WINDOWS-1251 < #{temp_in} > #{temp_out}"]
      )

      if status.success? && File.exists?(temp_out)
        File.read(temp_out)
      else
        content
      end
    ensure
      File.delete(temp_in) if File.exists?(temp_in)
      File.delete(temp_out) if File.exists?(temp_out)
    end
  end

  # Generate DEKLAR.TXT - Main VAT declaration
  # Format per PPDDS specification (590 chars + CRLF)
  private def generate_deklar(company : Company, vat_return : VatReturn) : String
    # VAT number WITH BG prefix (must match POKUPKI/PRODAJBI)
    vat_number = company.vat_number || ""
    vat_number = "BG#{vat_number}" unless vat_number.upcase.starts_with?("BG")
    period = "#{vat_return.period_year}#{vat_return.period_month.to_s.rjust(2, '0')}"

    # Representative info
    representative = vat_return.representative_name || company.authorized_person_name || company.manager_name || ""
    representative_egn = vat_return.representative_egn || company.authorized_person_egn || company.manager_egn || ""

    # 00-04: Submitter field (50 chars total): "EGN/Name" packed into 50
    submitter = "#{representative_egn}/#{representative}"

    # Compute totals for DEKLAR
    # *01-01: Total sales base = sum of all individual bases
    total_sales_base = vat_return.cell_01_01 + vat_return.cell_01_03 +
      vat_return.cell_01_05 + vat_return.cell_01_07 + vat_return.cell_01_11 +
      vat_return.cell_01_12 + vat_return.cell_01_13 + vat_return.cell_01_14 +
      vat_return.cell_01_16

    # *01-20: Total sales VAT = cell_01_40 (precomputed in calculate)
    total_sales_vat = vat_return.cell_01_40

    # 01-40: Total tax credit (purchase side)
    total_credit = vat_return.cell_01_22 + vat_return.cell_01_25

    String.build do |str|
      # Header section (121 chars)
      str << vat_number.ljust(15)                             # 00-01: ИН по ЗДДС с BG (15)
      str << company.name.ljust(50)[0, 50]                    # 00-02: Наименование (50)
      str << period                                           # 00-03: Данъчен период YYYYMM (6)
      str << submitter.ljust(50)[0, 50]                       # 00-04: Лице подаващо данните (50)

      # Counts (30 chars)
      str << vat_return.sales_count.to_s.rjust(15)            # 00-05: Брой документи продажби (15)
      str << vat_return.purchase_count.to_s.rjust(15)         # 00-06: Брой документи покупки (15)

      # Sales totals - 15 fields per spec (225 chars)
      str << fmt_amount(total_sales_base)                     # *01-01: Общ размер ДО (15.2)
      str << fmt_amount(total_sales_vat)                      # *01-20: Всичко начислен ДДС (15.2)
      str << fmt_amount(vat_return.cell_01_01)                # 01-11: ДО 20% (15.2)
      str << fmt_amount(vat_return.cell_01_02)                # 01-21: Начислен ДДС 20% (15.2)
      str << fmt_amount(vat_return.cell_01_03)                # 01-12: ДО ВОП и чл.82 (15.2)
      str << fmt_amount(vat_return.cell_01_04)                # 01-22: Начислен ДДС ВОП (15.2)
      str << fmt_amount(vat_return.cell_01_17)                # 01-23: Данък лични нужди (15.2)
      str << fmt_amount(vat_return.cell_01_05)                # 01-13: ДО 9% (15.2)
      str << fmt_amount(vat_return.cell_01_06)                # 01-24: Начислен ДДС 9% (15.2)
      str << fmt_amount(vat_return.cell_01_07)                # 01-14: ДО 0% глава 3 (15.2)
      str << fmt_amount(vat_return.cell_01_11)                # 01-15: ДО ВОД (15.2)
      str << fmt_amount(vat_return.cell_01_12)                # 01-16: ДО чл.140,146,173 (15.2)
      str << fmt_amount(vat_return.cell_01_13)                # 01-17: ДО услуги чл.21 ал.2 (15.2)
      str << fmt_amount(vat_return.cell_01_14)                # 01-18: ДО чл.69 ал.2 (15.2)
      str << fmt_amount(vat_return.cell_01_16)                # 01-19: ДО освободени (15.2)

      # Purchase totals - 6 fields per spec (90 chars)
      str << fmt_amount(vat_return.cell_01_20)                # 01-30: ДО без право/без ДК (15.2)
      str << fmt_amount(vat_return.cell_01_21)                # 01-31: ДО с пълен ДК (15.2)
      str << fmt_amount(vat_return.cell_01_22)                # 01-41: ДДС пълен ДК (15.2)
      str << fmt_amount(vat_return.cell_01_23)                # 01-32: ДО частичен ДК (15.2)
      str << fmt_amount(vat_return.cell_01_24)                # 01-42: ДДС частичен ДК (15.2)
      str << fmt_amount(vat_return.cell_01_25)                # 01-43: Год. корекция (15.2)

      # Result section - coefficient(4) + 8 amounts (124 chars)
      str << sprintf("%4.2f", 0.0)                            # 01-33: Коефициент чл.73 ал.5 (4)
      str << fmt_amount(total_credit)                         # 01-40: Общо данъчен кредит (15.2)
      str << fmt_amount(vat_return.cell_01_41)                # 01-50: ДДС за внасяне (15.2)
      str << fmt_amount(vat_return.cell_01_42)                # 01-60: ДДС за възстановяване (15.2)
      str << fmt_amount(vat_return.cell_01_50)                # 01-70: От предходен период (15.2)
      str << fmt_amount(vat_return.cell_01_43)                # 01-71: Ефективно внесен (15.2)
      str << fmt_amount(vat_return.cell_01_51)                # 01-80: За внасяне общо (15.2)
      str << fmt_amount(vat_return.cell_01_52)                # 01-81: За възстановяване общо (15.2)
      str << fmt_amount(0.0)                                  # 01-82: Възстановяване чл.92 ал.4 (15.2)

      str << CRLF
    end
  end

  # Generate POKUPKI.TXT - Purchase journal
  # Fixed-width format per ППЗДДС Приложение 10
  private def generate_pokupki(company : Company, vat_return : VatReturn) : String
    # VAT number WITH BG prefix (15 chars total)
    vat_number = company.vat_number || ""
    vat_number = "BG#{vat_number}" unless vat_number.upcase.starts_with?("BG")
    period = "#{vat_return.period_year}#{vat_return.period_month.to_s.rjust(2, '0')}"

    entries = VatJournalEntryQuery.new
      .vat_return_id(vat_return.id)
      .entry_type("purchase")
      .row_number.asc_order

    # Branch from company settings (vat_branch_number)
    branch = (company.vat_branch_number || 0).to_s

    String.build do |str|
      entries.each do |entry|
        str << vat_number.ljust(15)                           # 03-01: ИН по ЗДДС с BG (15)
        str << period.ljust(6)                                # 03-02: Данъчен период YYYYMM (6)
        str << branch.rjust(4)[0, 4]                          # 03-03: Клон/поделение (4, дясно подравнен)
        str << entry.row_number.to_s.rjust(15)                # 03-04: Пореден номер (15, дясно подравнен)
        str << entry.document_type.ljust(2)                   # 03-05: Вид документ (2)
        str << entry.document_number.ljust(20)[0, 20]         # 03-06: Номер документ (20)
        str << entry.formatted_date.ljust(10)                 # 03-07: Дата (10)
        str << (entry.counterpart_vat || "999999999999999").ljust(15)[0, 15]  # 03-08: ИН доставчик (15)
        str << (entry.counterpart_name || "").ljust(50)[0, 50]                # 03-09: Име доставчик (50)
        str << (entry.goods_description || "").ljust(30)[0, 30]               # 03-10: Вид стока (30)
        # Amount columns - 7 fields per ППЗДДС Приложение 10
        str << fmt_amount(entry.base_no_credit)               # 03-30: ДО без право на ДК (15.2)
        str << fmt_amount(entry.base_full_credit)             # 03-31: ДО с пълен ДК (15.2)
        str << fmt_amount(entry.vat_full_credit)              # 03-41: ДДС пълен ДК (15.2)
        str << fmt_amount(entry.base_partial_credit)          # 03-32: ДО частичен ДК (15.2)
        str << fmt_amount(entry.vat_partial_credit)           # 03-42: ДДС частичен ДК (15.2)
        str << fmt_amount(entry.annual_adjustment_base)       # 03-43: Год. корекция (15.2)
        str << fmt_amount(entry.vop_base)                     # 03-44: ДО тристранна операция (15.2)
        str << (entry.delivery_code || "  ").ljust(2)[0, 2]   # 03-45: Код доставка (2)
        str << CRLF
      end
    end
  end

  # Generate PRODAGBI.TXT - Sales journal
  # Fixed-width format per ППЗДДС Приложение 11
  private def generate_prodajbi(company : Company, vat_return : VatReturn) : String
    # VAT number WITH BG prefix (15 chars total)
    vat_number = company.vat_number || ""
    vat_number = "BG#{vat_number}" unless vat_number.upcase.starts_with?("BG")
    period = "#{vat_return.period_year}#{vat_return.period_month.to_s.rjust(2, '0')}"

    entries = VatJournalEntryQuery.new
      .vat_return_id(vat_return.id)
      .entry_type("sales")
      .row_number.asc_order

    # Branch from company settings (vat_branch_number)
    branch = (company.vat_branch_number || 0).to_s

    String.build do |str|
      entries.each do |entry|
        str << vat_number.ljust(15)                           # 02-01: ИН по ЗДДС с BG (15)
        str << period.ljust(6)                                # 02-02: Данъчен период YYYYMM (6)
        str << branch.rjust(4)[0, 4]                          # 02-03: Клон/поделение (4, дясно подравнен)
        str << entry.row_number.to_s.rjust(15)                # 02-04: Пореден номер (15, дясно подравнен)
        str << entry.document_type.ljust(2)                   # 02-05: Вид документ (2)
        str << entry.document_number.ljust(20)[0, 20]         # 02-06: Номер документ (20)
        str << entry.formatted_date.ljust(10)                 # 02-07: Дата (10)
        str << (entry.counterpart_vat || "999999999999999").ljust(15)[0, 15]  # 02-08: ИН клиент (15)
        str << (entry.counterpart_name || "").ljust(50)[0, 50]                # 02-09: Име клиент (50)
        str << (entry.goods_description || "").ljust(30)[0, 30]               # 02-10: Вид стока (30)
        # Amount columns - 17 fields per ППЗДДС Приложение 11
        str << fmt_amount(entry.total_base)                   # 02-10: Общ размер ДО (15.2)
        str << fmt_amount(entry.total_vat)                    # 02-20: Всичко начислен ДДС (15.2)
        str << fmt_amount(entry.sales_base_20)                # 02-11: ДО 20% (15.2)
        str << fmt_amount(entry.sales_vat_20)                 # 02-21: Начислен ДДС 20% (15.2)
        str << fmt_amount(entry.sales_base_vop)               # 02-12: ДО ВОП (15.2)
        str << fmt_amount(entry.sales_base_69_2_eu)           # 02-26: ДО чл.82 ал.2-5 (15.2)
        str << fmt_amount(entry.sales_vat_vop)                # 02-22: ДДС ВОП и чл.82 (15.2)
        str << fmt_amount(entry.sales_vat_personal)           # 02-23: Начислен данък лични нужди (15.2)
        str << fmt_amount(entry.sales_base_9)                 # 02-13: ДО 9% (15.2)
        str << fmt_amount(entry.sales_vat_9)                  # 02-24: Начислен ДДС 9% (15.2)
        str << fmt_amount(entry.sales_base_0_chapter3)        # 02-14: ДО 0% глава 3 (15.2)
        str << fmt_amount(entry.sales_base_vod)               # 02-15: ДО ВОД (15.2)
        str << fmt_amount(entry.sales_base_0_articles)        # 02-16: ДО чл.140,146,173 (15.2)
        str << fmt_amount(entry.sales_base_services_21)       # 02-17: ДО услуги чл.21 ал.2 (15.2)
        str << fmt_amount(entry.sales_base_69_2)              # 02-18: ДО чл.69 ал.2 (15.2)
        str << fmt_amount(entry.sales_base_exempt)            # 02-19: ДО освободени доставки (15.2)
        str << fmt_amount(entry.sales_base_vop_9)             # 02-25: ДО тристранна операция (15.2)
        str << (entry.delivery_code || "  ").ljust(2)[0, 2]   # 02-27: Код доставка (2)
        str << CRLF
      end
    end
  end

  # Format amount as fixed-width decimal (15 chars, 2 decimal places)
  private def fmt_amount(amount : Float64) : String
    sprintf("%15.2f", amount)
  end
end
