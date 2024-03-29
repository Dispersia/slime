struct Globals {
    trail_weight: f32,
    width: u32,
    height: u32,
};

struct TimeBuffer {
    time: u32,
    delta_time: f32,
};

struct SpeciesSetting {
    move_speed: f32,
    turn_speed: f32,
    sensor_angle_degrees: f32,
    sensor_offset_dst: f32,
    sensor_size: i32,
};

struct Agent {
    position: vec2<f32>,
    angle: f32,
};

struct Agents {
    agents: array<Agent>,
};

@group(0)
@binding(0)
var<uniform> globals: Globals;

@group(0)
@binding(1)
var<uniform> time: TimeBuffer;

@group(0)
@binding(2)
var<uniform> species_settings: SpeciesSetting;

@group(0)
@binding(3)
var<storage, read_write> agents: Agents;

@group(0)
@binding(4)
var trail_map_read: texture_storage_2d<rgba16float, read>;

@group(0)
@binding(5)
var trail_map_write: texture_storage_2d<rgba16float, write>;


struct ComputeInput {
    @builtin(global_invocation_id) global_invocation_id: vec3<u32>,
};

fn scale_to_range(state: f32) -> f32 {
    return state / 4294967295.0;
}

fn hash(state: u32) -> u32 {
    let state1 = state ^ 2747636419u;
    let state2 = state1 * 2654435769u;
    let state3 = state2 ^ state >> 16u;
    let state4 = state3 * 2654435769u;
    let state5 = state4 ^ state >> 16u;
    let state6 = state5 * 2654435769u;

    return state6;
}

fn sense(agent: Agent, sensor_angle_offset: f32) -> f32 {
    let sensor_angle = agent.angle + sensor_angle_offset;
    let sensor_dir = vec2<f32>(cos(sensor_angle), sin(sensor_angle));

    let sensor_pos = agent.position + sensor_dir * species_settings.sensor_offset_dst;

    let sensor_center_x = u32(sensor_pos.x);
    let sensor_center_y = u32(sensor_pos.y);

    var sum: f32 = 0.0;

    for(var offset_x: i32 = -species_settings.sensor_size; offset_x <= species_settings.sensor_size; offset_x = offset_x + 1) {
        for(var offset_y: i32 = -species_settings.sensor_size; offset_y <= species_settings.sensor_size; offset_y = offset_y + 1) {
            let sample_x = min(globals.width - 1u, max(0u, sensor_center_x + u32(offset_x)));
            let sample_y = min(globals.height - 1u, max(0u, sensor_center_y + u32(offset_y)));

            let current_map = textureLoad(trail_map_read, vec2<i32>(i32(sample_x), i32(sample_y)));
            let mask = vec4<f32>(1.0, 1.0, 1.0, 1.0) * 2.0 - 1.0;
            sum = sum + dot(mask, current_map);
        }
    }

    return sum;
}

@compute
@workgroup_size(64)
fn cs_main(input: ComputeInput) {
    let id = input.global_invocation_id;

    let total_agents = arrayLength(&agents.agents);
    let index = id.x;

    if (index >= total_agents) {
        return;
    }

    var agent: Agent = agents.agents[index];
    
    let random = hash(
        u32(agent.position.y) * globals.width
            + u32(agent.position.x)
            + hash(id.x + time.time * 100000u)
    );

    let sensor_angle_rad = species_settings.sensor_angle_degrees * (3.1415 / 180.0);
    let weight_forward = sense(agent, 0.0);
    let weight_left = sense(agent, sensor_angle_rad);
    let weight_right = sense(agent, -sensor_angle_rad);

    let random_steer_strength = scale_to_range(f32(random));
    let turn_speed = species_settings.turn_speed * 2.0 * 3.1415;

    if (weight_forward > weight_left && weight_forward > weight_right) {
        agents.agents[index].angle = agent.angle + 0.0;
    } else if (weight_forward < weight_left && weight_forward < weight_right) {
        agents.agents[index].angle = agent.angle + (random_steer_strength - 0.5) * 2.0 * turn_speed * time.delta_time;
    } else if (weight_right > weight_left) {
        agents.agents[index].angle = agent.angle - random_steer_strength * turn_speed * time.delta_time;
    } else if (weight_left > weight_right) {
        agents.agents[index].angle = agent.angle + random_steer_strength * turn_speed * time.delta_time;
    }

    let direction = vec2<f32>(cos(agent.angle), sin(agent.angle));
    var new_pos: vec2<f32> = agent.position + direction * time.delta_time * species_settings.move_speed;

    let global_width = f32(globals.width);
    let global_height = f32(globals.height);

    if (new_pos.x < 0.0 || new_pos.x >= global_width || new_pos.y < 0.0 || new_pos.y >= global_height) {
        let new_rand = hash(random);
        let random_angle = scale_to_range(f32(new_rand)) * 2.0 * 3.1415;

        new_pos.x = min(global_width - 1.0, max(0.0, new_pos.x));
        new_pos.y = min(global_height - 1.0, max(0.0, new_pos.y));
        agents.agents[index].angle = random_angle;
    } else {
        let current_pos = vec2<i32>(i32(new_pos.x), i32(new_pos.y));
        let current_map = textureLoad(trail_map_read, current_pos);
        
        textureStore(trail_map_write, current_pos, min(vec4<f32>(1.0, 1.0, 1.0, 1.0), current_map + vec4<f32>(1.0, 1.0, 1.0, 0.6) * globals.trail_weight * time.delta_time));
    }

    agents.agents[index].position = new_pos;
}
