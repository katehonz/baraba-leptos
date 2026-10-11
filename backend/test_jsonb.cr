# Test JSONB syntax
class TestJSONB < BaseModel
  table do
    column data : JSONB::Any = {} of String => JSON::Any
end
