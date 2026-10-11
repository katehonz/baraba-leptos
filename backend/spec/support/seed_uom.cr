def seed_uom(code : String = "C62")
  AppDatabase.exec <<-SQL
    INSERT INTO saft_units_of_measure (code, name_en, name_bg, symbol, is_common, sort_order, created_at, updated_at)
    VALUES ('#{code}', '#{code}', '#{code}', '#{code}', true, 1, NOW(), NOW())
    ON CONFLICT (code) DO NOTHING
  SQL
end

def seed_warehouse_doc_types
  AppDatabase.exec <<-SQL
    INSERT INTO warehouse_document_types
      (code, name_bg, saft_movement_type, needs_from, needs_to, default_is_incoming, system_only, sort_order, created_at, updated_at)
    VALUES
      ('RECEIPT','Приход','10',false,true,true,false,10,NOW(),NOW()),
      ('ISSUE','Разход','70',true,false,false,false,70,NOW(),NOW())
    ON CONFLICT (code) DO NOTHING
  SQL
end
