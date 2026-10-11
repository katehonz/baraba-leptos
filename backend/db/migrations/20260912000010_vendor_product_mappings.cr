class VendorProductMappings::V20260912000010 < Avram::Migrator::Migration::V1
  def migrate
    execute <<-SQL
      CREATE TABLE vendor_product_mappings (
        id BIGSERIAL PRIMARY KEY,
        company_id BIGINT NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
        counterpart_id BIGINT REFERENCES counterparts(id) ON DELETE SET NULL,
        vendor_name VARCHAR(255),
        vendor_sku VARCHAR(128),
        vendor_description TEXT,
        product_id BIGINT NOT NULL REFERENCES products(id) ON DELETE CASCADE,
        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
        updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
      )
      SQL
    execute "CREATE INDEX vendor_product_mappings_lookup_idx ON vendor_product_mappings (company_id, counterpart_id, vendor_sku)"
  end

  def rollback
    execute "DROP TABLE IF EXISTS vendor_product_mappings"
  end
end
