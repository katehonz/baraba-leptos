struct FsCatalogLine
  include JSON::Serializable

  getter code : String
  getter parent : String?
  getter kind : String
  getter side : String
  getter name : String
end

# Runtime path is CWD-relative `data/fs/*.json` (Docker WORKDIR /app, lucky from backend/).
module FsCatalog
  extend self

  STATEMENTS = %w[balance pl cashflow equity]
  KINDS      = %w[header line total]

  @@lines = {} of String => Array(FsCatalogLine)
  @@defaults = {} of String => Hash(String, String)

  def clear_cache : Nil
    @@lines.clear
    @@defaults.clear
  end

  def lines(statement : String) : Array(FsCatalogLine)
    unless STATEMENTS.includes?(statement)
      raise ArgumentError.new("непознат отчет #{statement}")
    end
    @@lines[statement] ||= load_lines(statement)
  end

  def defaults(statement : String) : Hash(String, String)
    unless STATEMENTS.includes?(statement)
      raise ArgumentError.new("непознат отчет #{statement}")
    end
    @@defaults[statement] ||= load_defaults(statement)
  end

  def has_code?(statement : String, code : String) : Bool
    lines(statement).any? { |row| row.code == code }
  end

  def find(statement : String, code : String) : FsCatalogLine?
    lines(statement).find { |row| row.code == code }
  end

  def children(statement : String, parent_code : String?) : Array(FsCatalogLine)
    key = blank_to_nil(parent_code)
    lines(statement).select { |row| blank_to_nil(row.parent) == key }
  end

  private def blank_to_nil(value : String?) : String?
    return nil if value.nil? || value.blank?
    value
  end

  def data_path(filename : String) : String
    File.join("data", "fs", filename)
  end

  private def load_lines(statement : String) : Array(FsCatalogLine)
    path = data_path("#{statement}.json")
    raise "липсва каталог #{path}" unless File.exists?(path)
    Array(FsCatalogLine).from_json(File.read(path))
  end

  private def load_defaults(statement : String) : Hash(String, String)
    path = data_path("defaults_#{statement}.json")
    raise "липсват стандартни формули #{path}" unless File.exists?(path)
    Hash(String, String).from_json(File.read(path))
  end
end
