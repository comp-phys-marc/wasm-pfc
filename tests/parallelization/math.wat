(module
    (func $dot (param $v0 i8) (param $v1 i8) (result i8)
        
    )
    (func $dot_three (param $vx i8) (param $vy i8) (param $vz i8) (param $wx i8) (param $wy i8) (param $wz i8) (result i8)
        (i8.add
            (i8.mul
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
    (func $dot_two (param $vx i8) (param $vy i8) (param $wx i8) (param $wy i8) (result i8)
        (i8.add
            (i8.mul
                (get_local $vx)
                (get_local $wx)
            )
            (i8.mul
                (get_local $vy)
                (get_local $wy)
            )
        )
    )
    (export "dot_three" (func $dot_three))
    (export "dot_two" (func $dot_two))
)
