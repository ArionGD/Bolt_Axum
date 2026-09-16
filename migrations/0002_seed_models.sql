-- 0002_seed_models.sql: EV Scooty & E-Rickshaw models seed data

insert into vehicle_models (
  brand, model_name, variant, body_type, battery_kwh, range_km, motor_power_kw, charging_ac_kw, charging_dc_kw, seating_capacity, ex_showroom_price
) values
  -- Scooty models (separate models in Scooty)
  ('EV Scooty', 'Scooty Model 1', 'Standard 2.0 kWh', 'Scooty', 2.00, 85, 2.50, 0.60, 1.20, 2, 65000.00),
  ('EV Scooty', 'Scooty Model 2', 'City 2.5 kWh', 'Scooty', 2.50, 105, 3.20, 0.80, 1.50, 2, 78000.00),
  ('EV Scooty', 'Scooty Model Pro', 'Long Range 3.2 kWh', 'Scooty', 3.20, 130, 4.00, 1.00, 2.00, 2, 92000.00),

  -- E-Rickshaw (single model)
  ('EV Rickshaw', 'E-Rickshaw Model 1', 'Passenger 5-Seater L5M', 'E-Rickshaw', 4.80, 110, 4.50, 1.20, 3.00, 5, 145000.00)
on conflict (brand, model_name, variant) do nothing;
