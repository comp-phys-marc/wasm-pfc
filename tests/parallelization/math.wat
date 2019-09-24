(module
    (func $dot (param $v0 i32) (param $v1 i32) (result i32)
        
    )
    (func $dot_three (param $vx i32) (param $vy i32) (param $vz i32) (param $wx i32) (param $wy i32) (param $wz i32) (result i32)
        (i32.add
            (i32.mul
                (get_local $vx)
                (get_local $wx)
            )
            (call $dot_two
                (get_local $vy)
                (get_local $vz)
                (get_local $wy)
                (get_local $wz)
            )
        )
    )
    (func $dot_two (param $vx i32) (param $vy i32) (param $wx i32) (param $wy i32) (result i32)
        (i32.add
            (i32.mul
                (get_local $vx)
                (get_local $wx)
            )
            (i32.mul
                (get_local $vy)
                (get_local $wy)
            )
        )
    )
    (export "dot_three" (func $dot_three))
    (export "dot_two" (func $dot_two))
)

