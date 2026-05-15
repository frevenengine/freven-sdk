use freven_world_guest_sdk::{
    BlockDescriptor, InitialWorldSpawnHint, LifecycleResult, SectionY, StartContext, TickContext,
    WorldGenCallResult, WorldGenColumnBuilder, WorldGenContext,
};

const GUEST_ID: &str = "example.first_wasm";
const BLOCK_KEY: &str = "example.first_wasm:ground";
const WORLDGEN_KEY: &str = "example.first_wasm:flat";

fn ground_block() -> BlockDescriptor {
    BlockDescriptor::solid_colored_cube(0x3C7A_52FF)
}

fn start_server(ctx: StartContext<'_>) -> LifecycleResult {
    freven_world_guest_sdk::log_info!(
        "first wasm mod started experience={} mod={}",
        ctx.experience_id(),
        ctx.mod_id()
    );
    LifecycleResult::default()
}

fn tick_server(ctx: TickContext<'_>) -> LifecycleResult {
    if ctx.tick().is_multiple_of(120) {
        freven_world_guest_sdk::log_info!("first wasm mod heartbeat tick={}", ctx.tick());
    }
    LifecycleResult::default()
}

fn generate_worldgen(ctx: WorldGenContext<'_>) -> WorldGenCallResult {
    let ground = ctx
        .init()
        .block_id_by_key(BLOCK_KEY)
        .expect("first wasm worldgen requires its registered ground block");

    let mut column = WorldGenColumnBuilder::for_request(ctx.request());
    column.fill_section(SectionY::new(0), ground);

    let mut output = column.finish();

    if ctx.request().column.cx == 0 && ctx.request().column.cz == 0 {
        output.bootstrap.initial_world_spawn_hint = Some(InitialWorldSpawnHint {
            feet_position: [16.5, 32.0, 16.5],
        });
    }

    WorldGenCallResult { output }
}

freven_world_guest_sdk::wasm_guest!(
    guest_id: GUEST_ID,
    registration: {
        block: BLOCK_KEY => ground_block(),
        worldgen: WORLDGEN_KEY => generate_worldgen,
    },
    lifecycle: {
        start_server: start_server,
        tick_server: tick_server,
    },
);
