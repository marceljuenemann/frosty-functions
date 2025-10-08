export namespace ic {
  export function time(): i64 {
    return sys_internal_ic_time();
  }

  export function caller(): string {
    // Implementation would use host functions to get caller
    return "anonymous"; // placeholder
  }

  export function randomInt(max: i32): i32 {
    return sys_internal_random() % max;
  }
}

// Internal host function declarations
@external("env", "ic_time_host")
declare function sys_internal_ic_time(): i64;

@external("env", "ic_random_host")
declare function sys_internal_random(): i32;
