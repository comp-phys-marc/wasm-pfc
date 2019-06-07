(module
    (func $calc (param $vi i32) (param $vf i32) (param $t i32) (param $torque i32) (param $velocity i32) (result i32)
        (i32.div_u
            (call $accel
                (get_local $vf)
                (get_local $vi)
                (get_local $t)
            )
            (call $power
                (get_local $torque)
                (get_local $velocity)
            )
        )
    )
    (func $accel (param $vi i32) (param $vf i32) (param $t i32) (result i32)
        (i32.div_u
            (i32.sub
                (get_local $vf)
                (get_local $vi)
            )
            (get_local $t)
        )
    )
    (func $power (param $torque i32) (param $velocity i32) (result i32)
        (i32.mul
            (get_local $torque)
            (get_local $velocity)
        )
    )
    (export "calc" (func $calc))
)
