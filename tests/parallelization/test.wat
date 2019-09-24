(module
    (func $calc (param $vi i32) (param $vf i32) (param $t i32) (param $torque i32) (param $velocity i32) (result i32)
        (i32.lt_u
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
            (get_local $torque)
        )
        (if (result i32)
            (then
                (block (result i32)
                    (call $accel
                        (get_local $vi)
                        (get_local $vf)
                        (get_local $t)
                    )
                )
            )
            (else
                (i32.const 1)
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
    (func $invPower (param $torque i32) (param $velocity i32) (result i32)
        (i32.div_u
            (i32.const 1)
            (i32.mul
                (get_local $torque)
                (get_local $velocity)
            )
        )
    )
    (export "calc" (func $calc))
)
