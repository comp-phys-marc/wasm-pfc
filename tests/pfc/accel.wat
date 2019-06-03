(module
    (func $accel (param $vi i32) (param $vf i32) (param $t i32) (result i32)
        (i32.div_u
            (i32.sub
                (get_local $vf)
                (get_local $vi)
            )
            (get_local $t)
        )
    )
    (export "accel" (func $accel))
)
