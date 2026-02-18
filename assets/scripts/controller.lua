-- controller.lua — Default movement controller for Saturday Morning Engine
-- Lua provides intents (desired motion), Rust resolves physics/collision.

function on_init()
    -- Called on script load/reload. Use for one-time setup.
end

function on_update(dt)
    local move_x = 0
    if engine.input.is_held("left") or engine.input.is_held("a") then
        move_x = move_x - 1
    end
    if engine.input.is_held("right") or engine.input.is_held("d") then
        move_x = move_x + 1
    end

    local jump = engine.input.is_just_pressed("space")
        or engine.input.is_just_pressed("w")
        or engine.input.is_just_pressed("up")

    engine.actor.set_intent(move_x, jump)
end
