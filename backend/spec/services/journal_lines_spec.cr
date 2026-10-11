require "../spec_helper"

describe JournalLines do
  it "parses debit and credit from float, int, and string JSON values" do
    lines = JournalLines.parse(%([
      {"debit":3600.0,"credit":0.0},
      {"debit":1500,"credit":0},
      {"debit":"200.5","credit":"10"},
      {"debit_amount":80.0,"credit_amount":0.0}
    ]))
    lines.size.should eq(4)
    JournalLines.debit(lines[0]).should eq(3600.0)
    JournalLines.debit(lines[1]).should eq(1500.0)
    JournalLines.debit(lines[2]).should eq(200.5)
    JournalLines.credit(lines[2]).should eq(10.0)
    JournalLines.debit(lines[3]).should eq(80.0)
  end

  it "matches a salary line by account_code when account_id is missing" do
    line = JournalLines.parse(%([{"account_code":"604","debit":3600.0,"credit":0.0}])).first
    JournalLines.account_id(line).should be_nil
    JournalLines.account_code(line).should eq("604")
    JournalLines.matches_account?(line, 603_i64, "604").should be_true
    JournalLines.matches_account?(line, 603_i64, "421").should be_false
  end

  it "resolves chart account id from account_code" do
    line = JournalLines.parse(%([{"account_code":"604","debit":3600.0,"credit":0.0}])).first
    known = {} of Int64 => String
    known[603_i64] = "604"
    by_code = {"604" => 603_i64}
    JournalLines.resolve_account_id(line, known, by_code).should eq(603_i64)
  end

  it "prefers a valid account_id over account_code" do
    line = JournalLines.parse(%([{"account_id":77,"account_code":"604","debit":1.0,"credit":0.0}])).first
    known = {} of Int64 => String
    known[77_i64] = "401"
    by_code = {"604" => 603_i64}
    JournalLines.resolve_account_id(line, known, by_code).should eq(77_i64)
  end

  it "sums document total from debit lines, not the stored tax base" do
    lines = %([
      {"account_code":"602","debit":171.52,"credit":0.0},
      {"account_code":"4531","debit":34.30,"credit":0.0},
      {"account_code":"401","debit":0.0,"credit":205.82}
    ])
    JournalLines.total_debit(lines).should be_close(205.82, 0.001)
    JournalLines.total_credit(lines).should be_close(205.82, 0.001)
    JournalLines.document_total(lines, 171.52).should be_close(205.82, 0.001)
  end

  it "falls back to stored total when lines have no amounts" do
    JournalLines.document_total("[]", 100.0).should eq(100.0)
    JournalLines.document_total(nil, 50.0).should eq(50.0)
    JournalLines.document_total("[]", nil).should eq(0.0)
  end

  it "reads JSON-column totals when journal_lines table is empty" do
    user = UserFactory.create
    company = SaveCompany.create!(name: "JE Lines ООД", eik: "555666777")
    entry = SaveJournalEntry.create!(
      company_id: company.id,
      entry_date: Time.utc(2026, 3, 15),
      description: "Касова бележка бензиностанция",
      status: "draft",
      lines: %([
        {"account_code":"602","debit":83.33,"credit":0.0},
        {"account_code":"4531","debit":16.67,"credit":0.0},
        {"account_code":"401","debit":0.0,"credit":100.0}
      ])
    )

    totals = JournalLines.totals_for(entry)
    totals[:count].should eq(3)
    totals[:debit].should be_close(100.0, 0.001)
    totals[:credit].should be_close(100.0, 0.001)
  end

  it "resolves account_code from account_id when the line code is blank" do
    line = JournalLines.parse(%([{"account_id":1209,"account_code":"","debit":2700,"credit":0}])).first
    id_to_code = {1209_i64 => "401"}
    JournalLines.account_code(line).should be_nil
    JournalLines.resolve_account_code(line, id_to_code).should eq("401")
    JournalLines.debit(line).should eq(2700.0)
  end

  it "fills missing account_code from the chart" do
    company = SaveCompany.create!(name: "Кодове ООД", eik: "555666778")
    acc = SaveAccount.create!(company_id: company.id, code: "401", name: "Доставчици", account_type: "liability")
    lines = %([{"account_id":#{acc.id},"account_code":"","debit":100.0,"credit":0.0}])
    enriched = JournalLines.enrich_account_codes(lines, company.id)
    parsed = JournalLines.parse(enriched).first
    JournalLines.account_code(parsed).should eq("401")
  end
end
