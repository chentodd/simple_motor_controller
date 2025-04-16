pub mod motor_ {
    pub mod MotorRx_ {
        #[derive(Debug, Default, PartialEq, Clone)]
        pub struct _Hazzer([u8; 1]);
        impl _Hazzer {
            ///Query presence of `target_dist`
            #[inline]
            pub fn r#target_dist(&self) -> bool {
                (self.0[0] & 1) != 0
            }
            ///Set presence of `target_dist`
            #[inline]
            pub fn set_target_dist(&mut self) {
                let elem = &mut self.0[0];
                *elem |= 1;
            }
            ///Clear presence of `target_dist`
            #[inline]
            pub fn clear_target_dist(&mut self) {
                let elem = &mut self.0[0];
                *elem &= !1;
            }
            ///Builder method that sets the presence of `target_dist`. Useful for initializing the Hazzer.
            #[inline]
            pub fn init_target_dist(mut self) -> Self {
                self.set_target_dist();
                self
            }
            ///Query presence of `target_vel`
            #[inline]
            pub fn r#target_vel(&self) -> bool {
                (self.0[0] & 2) != 0
            }
            ///Set presence of `target_vel`
            #[inline]
            pub fn set_target_vel(&mut self) {
                let elem = &mut self.0[0];
                *elem |= 2;
            }
            ///Clear presence of `target_vel`
            #[inline]
            pub fn clear_target_vel(&mut self) {
                let elem = &mut self.0[0];
                *elem &= !2;
            }
            ///Builder method that sets the presence of `target_vel`. Useful for initializing the Hazzer.
            #[inline]
            pub fn init_target_vel(mut self) -> Self {
                self.set_target_vel();
                self
            }
            ///Query presence of `target_vel_end`
            #[inline]
            pub fn r#target_vel_end(&self) -> bool {
                (self.0[0] & 4) != 0
            }
            ///Set presence of `target_vel_end`
            #[inline]
            pub fn set_target_vel_end(&mut self) {
                let elem = &mut self.0[0];
                *elem |= 4;
            }
            ///Clear presence of `target_vel_end`
            #[inline]
            pub fn clear_target_vel_end(&mut self) {
                let elem = &mut self.0[0];
                *elem &= !4;
            }
            ///Builder method that sets the presence of `target_vel_end`. Useful for initializing the Hazzer.
            #[inline]
            pub fn init_target_vel_end(mut self) -> Self {
                self.set_target_vel_end();
                self
            }
            ///Query presence of `kp`
            #[inline]
            pub fn r#kp(&self) -> bool {
                (self.0[0] & 8) != 0
            }
            ///Set presence of `kp`
            #[inline]
            pub fn set_kp(&mut self) {
                let elem = &mut self.0[0];
                *elem |= 8;
            }
            ///Clear presence of `kp`
            #[inline]
            pub fn clear_kp(&mut self) {
                let elem = &mut self.0[0];
                *elem &= !8;
            }
            ///Builder method that sets the presence of `kp`. Useful for initializing the Hazzer.
            #[inline]
            pub fn init_kp(mut self) -> Self {
                self.set_kp();
                self
            }
            ///Query presence of `ki`
            #[inline]
            pub fn r#ki(&self) -> bool {
                (self.0[0] & 16) != 0
            }
            ///Set presence of `ki`
            #[inline]
            pub fn set_ki(&mut self) {
                let elem = &mut self.0[0];
                *elem |= 16;
            }
            ///Clear presence of `ki`
            #[inline]
            pub fn clear_ki(&mut self) {
                let elem = &mut self.0[0];
                *elem &= !16;
            }
            ///Builder method that sets the presence of `ki`. Useful for initializing the Hazzer.
            #[inline]
            pub fn init_ki(mut self) -> Self {
                self.set_ki();
                self
            }
            ///Query presence of `kd`
            #[inline]
            pub fn r#kd(&self) -> bool {
                (self.0[0] & 32) != 0
            }
            ///Set presence of `kd`
            #[inline]
            pub fn set_kd(&mut self) {
                let elem = &mut self.0[0];
                *elem |= 32;
            }
            ///Clear presence of `kd`
            #[inline]
            pub fn clear_kd(&mut self) {
                let elem = &mut self.0[0];
                *elem &= !32;
            }
            ///Builder method that sets the presence of `kd`. Useful for initializing the Hazzer.
            #[inline]
            pub fn init_kd(mut self) -> Self {
                self.set_kd();
                self
            }
        }
    }
    #[derive(Debug, PartialEq, Clone)]
    pub struct MotorRx {
        pub r#operation: Operation,
        pub r#target_dist: f32,
        pub r#target_vel: f32,
        pub r#target_vel_end: f32,
        pub r#kp: f32,
        pub r#ki: f32,
        pub r#kd: f32,
        pub _has: MotorRx_::_Hazzer,
    }
    impl ::core::default::Default for MotorRx {
        fn default() -> Self {
            Self {
                r#operation: ::core::default::Default::default(),
                r#target_dist: ::core::default::Default::default(),
                r#target_vel: ::core::default::Default::default(),
                r#target_vel_end: ::core::default::Default::default(),
                r#kp: ::core::default::Default::default(),
                r#ki: ::core::default::Default::default(),
                r#kd: ::core::default::Default::default(),
                _has: ::core::default::Default::default(),
            }
        }
    }
    impl MotorRx {
        ///Return a reference to `target_dist` as an `Option`
        #[inline]
        pub fn r#target_dist(&self) -> ::core::option::Option<&f32> {
            self._has.r#target_dist().then_some(&self.r#target_dist)
        }
        ///Return a mutable reference to `target_dist` as an `Option`
        #[inline]
        pub fn mut_target_dist(&mut self) -> ::core::option::Option<&mut f32> {
            self._has.r#target_dist().then_some(&mut self.r#target_dist)
        }
        ///Set the value and presence of `target_dist`
        #[inline]
        pub fn set_target_dist(&mut self, value: f32) {
            self._has.set_target_dist();
            self.r#target_dist = value.into();
        }
        ///Clear the presence of `target_dist`
        #[inline]
        pub fn clear_target_dist(&mut self) {
            self._has.clear_target_dist();
        }
        ///Return a reference to `target_vel` as an `Option`
        #[inline]
        pub fn r#target_vel(&self) -> ::core::option::Option<&f32> {
            self._has.r#target_vel().then_some(&self.r#target_vel)
        }
        ///Return a mutable reference to `target_vel` as an `Option`
        #[inline]
        pub fn mut_target_vel(&mut self) -> ::core::option::Option<&mut f32> {
            self._has.r#target_vel().then_some(&mut self.r#target_vel)
        }
        ///Set the value and presence of `target_vel`
        #[inline]
        pub fn set_target_vel(&mut self, value: f32) {
            self._has.set_target_vel();
            self.r#target_vel = value.into();
        }
        ///Clear the presence of `target_vel`
        #[inline]
        pub fn clear_target_vel(&mut self) {
            self._has.clear_target_vel();
        }
        ///Return a reference to `target_vel_end` as an `Option`
        #[inline]
        pub fn r#target_vel_end(&self) -> ::core::option::Option<&f32> {
            self._has.r#target_vel_end().then_some(&self.r#target_vel_end)
        }
        ///Return a mutable reference to `target_vel_end` as an `Option`
        #[inline]
        pub fn mut_target_vel_end(&mut self) -> ::core::option::Option<&mut f32> {
            self._has.r#target_vel_end().then_some(&mut self.r#target_vel_end)
        }
        ///Set the value and presence of `target_vel_end`
        #[inline]
        pub fn set_target_vel_end(&mut self, value: f32) {
            self._has.set_target_vel_end();
            self.r#target_vel_end = value.into();
        }
        ///Clear the presence of `target_vel_end`
        #[inline]
        pub fn clear_target_vel_end(&mut self) {
            self._has.clear_target_vel_end();
        }
        ///Return a reference to `kp` as an `Option`
        #[inline]
        pub fn r#kp(&self) -> ::core::option::Option<&f32> {
            self._has.r#kp().then_some(&self.r#kp)
        }
        ///Return a mutable reference to `kp` as an `Option`
        #[inline]
        pub fn mut_kp(&mut self) -> ::core::option::Option<&mut f32> {
            self._has.r#kp().then_some(&mut self.r#kp)
        }
        ///Set the value and presence of `kp`
        #[inline]
        pub fn set_kp(&mut self, value: f32) {
            self._has.set_kp();
            self.r#kp = value.into();
        }
        ///Clear the presence of `kp`
        #[inline]
        pub fn clear_kp(&mut self) {
            self._has.clear_kp();
        }
        ///Return a reference to `ki` as an `Option`
        #[inline]
        pub fn r#ki(&self) -> ::core::option::Option<&f32> {
            self._has.r#ki().then_some(&self.r#ki)
        }
        ///Return a mutable reference to `ki` as an `Option`
        #[inline]
        pub fn mut_ki(&mut self) -> ::core::option::Option<&mut f32> {
            self._has.r#ki().then_some(&mut self.r#ki)
        }
        ///Set the value and presence of `ki`
        #[inline]
        pub fn set_ki(&mut self, value: f32) {
            self._has.set_ki();
            self.r#ki = value.into();
        }
        ///Clear the presence of `ki`
        #[inline]
        pub fn clear_ki(&mut self) {
            self._has.clear_ki();
        }
        ///Return a reference to `kd` as an `Option`
        #[inline]
        pub fn r#kd(&self) -> ::core::option::Option<&f32> {
            self._has.r#kd().then_some(&self.r#kd)
        }
        ///Return a mutable reference to `kd` as an `Option`
        #[inline]
        pub fn mut_kd(&mut self) -> ::core::option::Option<&mut f32> {
            self._has.r#kd().then_some(&mut self.r#kd)
        }
        ///Set the value and presence of `kd`
        #[inline]
        pub fn set_kd(&mut self, value: f32) {
            self._has.set_kd();
            self.r#kd = value.into();
        }
        ///Clear the presence of `kd`
        #[inline]
        pub fn clear_kd(&mut self) {
            self._has.clear_kd();
        }
    }
    impl ::micropb::MessageDecode for MotorRx {
        fn decode<IMPL_MICROPB_READ: ::micropb::PbRead>(
            &mut self,
            decoder: &mut ::micropb::PbDecoder<IMPL_MICROPB_READ>,
            len: usize,
        ) -> Result<(), ::micropb::DecodeError<IMPL_MICROPB_READ::Error>> {
            use ::micropb::{PbVec, PbMap, PbString, FieldDecode};
            let before = decoder.bytes_read();
            while decoder.bytes_read() - before < len {
                let tag = decoder.decode_tag()?;
                match tag.field_num() {
                    0 => return Err(::micropb::DecodeError::ZeroField),
                    1u32 => {
                        let mut_ref = &mut self.r#operation;
                        {
                            let val = decoder.decode_int32().map(|n| Operation(n as _))?;
                            let val_ref = &val;
                            if val_ref.0 != 0 {
                                *mut_ref = val as _;
                            }
                        };
                    }
                    2u32 => {
                        let mut_ref = &mut self.r#target_dist;
                        {
                            let val = decoder.decode_float()?;
                            *mut_ref = val as _;
                        };
                        self._has.set_target_dist();
                    }
                    3u32 => {
                        let mut_ref = &mut self.r#target_vel;
                        {
                            let val = decoder.decode_float()?;
                            *mut_ref = val as _;
                        };
                        self._has.set_target_vel();
                    }
                    4u32 => {
                        let mut_ref = &mut self.r#target_vel_end;
                        {
                            let val = decoder.decode_float()?;
                            *mut_ref = val as _;
                        };
                        self._has.set_target_vel_end();
                    }
                    5u32 => {
                        let mut_ref = &mut self.r#kp;
                        {
                            let val = decoder.decode_float()?;
                            *mut_ref = val as _;
                        };
                        self._has.set_kp();
                    }
                    6u32 => {
                        let mut_ref = &mut self.r#ki;
                        {
                            let val = decoder.decode_float()?;
                            *mut_ref = val as _;
                        };
                        self._has.set_ki();
                    }
                    7u32 => {
                        let mut_ref = &mut self.r#kd;
                        {
                            let val = decoder.decode_float()?;
                            *mut_ref = val as _;
                        };
                        self._has.set_kd();
                    }
                    _ => {
                        decoder.skip_wire_value(tag.wire_type())?;
                    }
                }
            }
            Ok(())
        }
    }
    impl ::micropb::MessageEncode for MotorRx {
        fn encode<IMPL_MICROPB_WRITE: ::micropb::PbWrite>(
            &self,
            encoder: &mut ::micropb::PbEncoder<IMPL_MICROPB_WRITE>,
        ) -> Result<(), IMPL_MICROPB_WRITE::Error> {
            use ::micropb::{PbVec, PbMap, PbString, FieldEncode};
            {
                let val_ref = &self.r#operation;
                if val_ref.0 != 0 {
                    encoder.encode_varint32(8u32)?;
                    encoder.encode_int32(val_ref.0 as _)?;
                }
            }
            {
                if let Some(val_ref) = self.r#target_dist() {
                    encoder.encode_varint32(21u32)?;
                    encoder.encode_float(*val_ref)?;
                }
            }
            {
                if let Some(val_ref) = self.r#target_vel() {
                    encoder.encode_varint32(29u32)?;
                    encoder.encode_float(*val_ref)?;
                }
            }
            {
                if let Some(val_ref) = self.r#target_vel_end() {
                    encoder.encode_varint32(37u32)?;
                    encoder.encode_float(*val_ref)?;
                }
            }
            {
                if let Some(val_ref) = self.r#kp() {
                    encoder.encode_varint32(45u32)?;
                    encoder.encode_float(*val_ref)?;
                }
            }
            {
                if let Some(val_ref) = self.r#ki() {
                    encoder.encode_varint32(53u32)?;
                    encoder.encode_float(*val_ref)?;
                }
            }
            {
                if let Some(val_ref) = self.r#kd() {
                    encoder.encode_varint32(61u32)?;
                    encoder.encode_float(*val_ref)?;
                }
            }
            Ok(())
        }
        fn compute_size(&self) -> usize {
            use ::micropb::{PbVec, PbMap, PbString, FieldEncode};
            let mut size = 0;
            {
                let val_ref = &self.r#operation;
                if val_ref.0 != 0 {
                    size += 1usize + ::micropb::size::sizeof_int32(val_ref.0 as _);
                }
            }
            {
                if let Some(val_ref) = self.r#target_dist() {
                    size += 1usize + 4;
                }
            }
            {
                if let Some(val_ref) = self.r#target_vel() {
                    size += 1usize + 4;
                }
            }
            {
                if let Some(val_ref) = self.r#target_vel_end() {
                    size += 1usize + 4;
                }
            }
            {
                if let Some(val_ref) = self.r#kp() {
                    size += 1usize + 4;
                }
            }
            {
                if let Some(val_ref) = self.r#ki() {
                    size += 1usize + 4;
                }
            }
            {
                if let Some(val_ref) = self.r#kd() {
                    size += 1usize + 4;
                }
            }
            size
        }
    }
    pub mod MotorTx_ {
        #[derive(Debug, Default, PartialEq, Clone)]
        pub struct _Hazzer([u8; 1]);
        impl _Hazzer {
            ///Query presence of `intp_pos`
            #[inline]
            pub fn r#intp_pos(&self) -> bool {
                (self.0[0] & 1) != 0
            }
            ///Set presence of `intp_pos`
            #[inline]
            pub fn set_intp_pos(&mut self) {
                let elem = &mut self.0[0];
                *elem |= 1;
            }
            ///Clear presence of `intp_pos`
            #[inline]
            pub fn clear_intp_pos(&mut self) {
                let elem = &mut self.0[0];
                *elem &= !1;
            }
            ///Builder method that sets the presence of `intp_pos`. Useful for initializing the Hazzer.
            #[inline]
            pub fn init_intp_pos(mut self) -> Self {
                self.set_intp_pos();
                self
            }
            ///Query presence of `intp_vel`
            #[inline]
            pub fn r#intp_vel(&self) -> bool {
                (self.0[0] & 2) != 0
            }
            ///Set presence of `intp_vel`
            #[inline]
            pub fn set_intp_vel(&mut self) {
                let elem = &mut self.0[0];
                *elem |= 2;
            }
            ///Clear presence of `intp_vel`
            #[inline]
            pub fn clear_intp_vel(&mut self) {
                let elem = &mut self.0[0];
                *elem &= !2;
            }
            ///Builder method that sets the presence of `intp_vel`. Useful for initializing the Hazzer.
            #[inline]
            pub fn init_intp_vel(mut self) -> Self {
                self.set_intp_vel();
                self
            }
            ///Query presence of `intp_acc`
            #[inline]
            pub fn r#intp_acc(&self) -> bool {
                (self.0[0] & 4) != 0
            }
            ///Set presence of `intp_acc`
            #[inline]
            pub fn set_intp_acc(&mut self) {
                let elem = &mut self.0[0];
                *elem |= 4;
            }
            ///Clear presence of `intp_acc`
            #[inline]
            pub fn clear_intp_acc(&mut self) {
                let elem = &mut self.0[0];
                *elem &= !4;
            }
            ///Builder method that sets the presence of `intp_acc`. Useful for initializing the Hazzer.
            #[inline]
            pub fn init_intp_acc(mut self) -> Self {
                self.set_intp_acc();
                self
            }
            ///Query presence of `intp_jerk`
            #[inline]
            pub fn r#intp_jerk(&self) -> bool {
                (self.0[0] & 8) != 0
            }
            ///Set presence of `intp_jerk`
            #[inline]
            pub fn set_intp_jerk(&mut self) {
                let elem = &mut self.0[0];
                *elem |= 8;
            }
            ///Clear presence of `intp_jerk`
            #[inline]
            pub fn clear_intp_jerk(&mut self) {
                let elem = &mut self.0[0];
                *elem &= !8;
            }
            ///Builder method that sets the presence of `intp_jerk`. Useful for initializing the Hazzer.
            #[inline]
            pub fn init_intp_jerk(mut self) -> Self {
                self.set_intp_jerk();
                self
            }
            ///Query presence of `actual_pos`
            #[inline]
            pub fn r#actual_pos(&self) -> bool {
                (self.0[0] & 16) != 0
            }
            ///Set presence of `actual_pos`
            #[inline]
            pub fn set_actual_pos(&mut self) {
                let elem = &mut self.0[0];
                *elem |= 16;
            }
            ///Clear presence of `actual_pos`
            #[inline]
            pub fn clear_actual_pos(&mut self) {
                let elem = &mut self.0[0];
                *elem &= !16;
            }
            ///Builder method that sets the presence of `actual_pos`. Useful for initializing the Hazzer.
            #[inline]
            pub fn init_actual_pos(mut self) -> Self {
                self.set_actual_pos();
                self
            }
            ///Query presence of `actual_vel`
            #[inline]
            pub fn r#actual_vel(&self) -> bool {
                (self.0[0] & 32) != 0
            }
            ///Set presence of `actual_vel`
            #[inline]
            pub fn set_actual_vel(&mut self) {
                let elem = &mut self.0[0];
                *elem |= 32;
            }
            ///Clear presence of `actual_vel`
            #[inline]
            pub fn clear_actual_vel(&mut self) {
                let elem = &mut self.0[0];
                *elem &= !32;
            }
            ///Builder method that sets the presence of `actual_vel`. Useful for initializing the Hazzer.
            #[inline]
            pub fn init_actual_vel(mut self) -> Self {
                self.set_actual_vel();
                self
            }
        }
    }
    #[derive(Debug, PartialEq, Clone)]
    pub struct MotorTx {
        pub r#operation_display: Operation,
        pub r#command_buffer_full: bool,
        pub r#intp_pos: f32,
        pub r#intp_vel: f32,
        pub r#intp_acc: f32,
        pub r#intp_jerk: f32,
        pub r#actual_pos: f32,
        pub r#actual_vel: f32,
        pub _has: MotorTx_::_Hazzer,
    }
    impl ::core::default::Default for MotorTx {
        fn default() -> Self {
            Self {
                r#operation_display: ::core::default::Default::default(),
                r#command_buffer_full: ::core::default::Default::default(),
                r#intp_pos: ::core::default::Default::default(),
                r#intp_vel: ::core::default::Default::default(),
                r#intp_acc: ::core::default::Default::default(),
                r#intp_jerk: ::core::default::Default::default(),
                r#actual_pos: ::core::default::Default::default(),
                r#actual_vel: ::core::default::Default::default(),
                _has: ::core::default::Default::default(),
            }
        }
    }
    impl MotorTx {
        ///Return a reference to `intp_pos` as an `Option`
        #[inline]
        pub fn r#intp_pos(&self) -> ::core::option::Option<&f32> {
            self._has.r#intp_pos().then_some(&self.r#intp_pos)
        }
        ///Return a mutable reference to `intp_pos` as an `Option`
        #[inline]
        pub fn mut_intp_pos(&mut self) -> ::core::option::Option<&mut f32> {
            self._has.r#intp_pos().then_some(&mut self.r#intp_pos)
        }
        ///Set the value and presence of `intp_pos`
        #[inline]
        pub fn set_intp_pos(&mut self, value: f32) {
            self._has.set_intp_pos();
            self.r#intp_pos = value.into();
        }
        ///Clear the presence of `intp_pos`
        #[inline]
        pub fn clear_intp_pos(&mut self) {
            self._has.clear_intp_pos();
        }
        ///Return a reference to `intp_vel` as an `Option`
        #[inline]
        pub fn r#intp_vel(&self) -> ::core::option::Option<&f32> {
            self._has.r#intp_vel().then_some(&self.r#intp_vel)
        }
        ///Return a mutable reference to `intp_vel` as an `Option`
        #[inline]
        pub fn mut_intp_vel(&mut self) -> ::core::option::Option<&mut f32> {
            self._has.r#intp_vel().then_some(&mut self.r#intp_vel)
        }
        ///Set the value and presence of `intp_vel`
        #[inline]
        pub fn set_intp_vel(&mut self, value: f32) {
            self._has.set_intp_vel();
            self.r#intp_vel = value.into();
        }
        ///Clear the presence of `intp_vel`
        #[inline]
        pub fn clear_intp_vel(&mut self) {
            self._has.clear_intp_vel();
        }
        ///Return a reference to `intp_acc` as an `Option`
        #[inline]
        pub fn r#intp_acc(&self) -> ::core::option::Option<&f32> {
            self._has.r#intp_acc().then_some(&self.r#intp_acc)
        }
        ///Return a mutable reference to `intp_acc` as an `Option`
        #[inline]
        pub fn mut_intp_acc(&mut self) -> ::core::option::Option<&mut f32> {
            self._has.r#intp_acc().then_some(&mut self.r#intp_acc)
        }
        ///Set the value and presence of `intp_acc`
        #[inline]
        pub fn set_intp_acc(&mut self, value: f32) {
            self._has.set_intp_acc();
            self.r#intp_acc = value.into();
        }
        ///Clear the presence of `intp_acc`
        #[inline]
        pub fn clear_intp_acc(&mut self) {
            self._has.clear_intp_acc();
        }
        ///Return a reference to `intp_jerk` as an `Option`
        #[inline]
        pub fn r#intp_jerk(&self) -> ::core::option::Option<&f32> {
            self._has.r#intp_jerk().then_some(&self.r#intp_jerk)
        }
        ///Return a mutable reference to `intp_jerk` as an `Option`
        #[inline]
        pub fn mut_intp_jerk(&mut self) -> ::core::option::Option<&mut f32> {
            self._has.r#intp_jerk().then_some(&mut self.r#intp_jerk)
        }
        ///Set the value and presence of `intp_jerk`
        #[inline]
        pub fn set_intp_jerk(&mut self, value: f32) {
            self._has.set_intp_jerk();
            self.r#intp_jerk = value.into();
        }
        ///Clear the presence of `intp_jerk`
        #[inline]
        pub fn clear_intp_jerk(&mut self) {
            self._has.clear_intp_jerk();
        }
        ///Return a reference to `actual_pos` as an `Option`
        #[inline]
        pub fn r#actual_pos(&self) -> ::core::option::Option<&f32> {
            self._has.r#actual_pos().then_some(&self.r#actual_pos)
        }
        ///Return a mutable reference to `actual_pos` as an `Option`
        #[inline]
        pub fn mut_actual_pos(&mut self) -> ::core::option::Option<&mut f32> {
            self._has.r#actual_pos().then_some(&mut self.r#actual_pos)
        }
        ///Set the value and presence of `actual_pos`
        #[inline]
        pub fn set_actual_pos(&mut self, value: f32) {
            self._has.set_actual_pos();
            self.r#actual_pos = value.into();
        }
        ///Clear the presence of `actual_pos`
        #[inline]
        pub fn clear_actual_pos(&mut self) {
            self._has.clear_actual_pos();
        }
        ///Return a reference to `actual_vel` as an `Option`
        #[inline]
        pub fn r#actual_vel(&self) -> ::core::option::Option<&f32> {
            self._has.r#actual_vel().then_some(&self.r#actual_vel)
        }
        ///Return a mutable reference to `actual_vel` as an `Option`
        #[inline]
        pub fn mut_actual_vel(&mut self) -> ::core::option::Option<&mut f32> {
            self._has.r#actual_vel().then_some(&mut self.r#actual_vel)
        }
        ///Set the value and presence of `actual_vel`
        #[inline]
        pub fn set_actual_vel(&mut self, value: f32) {
            self._has.set_actual_vel();
            self.r#actual_vel = value.into();
        }
        ///Clear the presence of `actual_vel`
        #[inline]
        pub fn clear_actual_vel(&mut self) {
            self._has.clear_actual_vel();
        }
    }
    impl ::micropb::MessageDecode for MotorTx {
        fn decode<IMPL_MICROPB_READ: ::micropb::PbRead>(
            &mut self,
            decoder: &mut ::micropb::PbDecoder<IMPL_MICROPB_READ>,
            len: usize,
        ) -> Result<(), ::micropb::DecodeError<IMPL_MICROPB_READ::Error>> {
            use ::micropb::{PbVec, PbMap, PbString, FieldDecode};
            let before = decoder.bytes_read();
            while decoder.bytes_read() - before < len {
                let tag = decoder.decode_tag()?;
                match tag.field_num() {
                    0 => return Err(::micropb::DecodeError::ZeroField),
                    1u32 => {
                        let mut_ref = &mut self.r#operation_display;
                        {
                            let val = decoder.decode_int32().map(|n| Operation(n as _))?;
                            let val_ref = &val;
                            if val_ref.0 != 0 {
                                *mut_ref = val as _;
                            }
                        };
                    }
                    2u32 => {
                        let mut_ref = &mut self.r#command_buffer_full;
                        {
                            let val = decoder.decode_bool()?;
                            let val_ref = &val;
                            if *val_ref {
                                *mut_ref = val as _;
                            }
                        };
                    }
                    3u32 => {
                        let mut_ref = &mut self.r#intp_pos;
                        {
                            let val = decoder.decode_float()?;
                            *mut_ref = val as _;
                        };
                        self._has.set_intp_pos();
                    }
                    4u32 => {
                        let mut_ref = &mut self.r#intp_vel;
                        {
                            let val = decoder.decode_float()?;
                            *mut_ref = val as _;
                        };
                        self._has.set_intp_vel();
                    }
                    5u32 => {
                        let mut_ref = &mut self.r#intp_acc;
                        {
                            let val = decoder.decode_float()?;
                            *mut_ref = val as _;
                        };
                        self._has.set_intp_acc();
                    }
                    6u32 => {
                        let mut_ref = &mut self.r#intp_jerk;
                        {
                            let val = decoder.decode_float()?;
                            *mut_ref = val as _;
                        };
                        self._has.set_intp_jerk();
                    }
                    7u32 => {
                        let mut_ref = &mut self.r#actual_pos;
                        {
                            let val = decoder.decode_float()?;
                            *mut_ref = val as _;
                        };
                        self._has.set_actual_pos();
                    }
                    8u32 => {
                        let mut_ref = &mut self.r#actual_vel;
                        {
                            let val = decoder.decode_float()?;
                            *mut_ref = val as _;
                        };
                        self._has.set_actual_vel();
                    }
                    _ => {
                        decoder.skip_wire_value(tag.wire_type())?;
                    }
                }
            }
            Ok(())
        }
    }
    impl ::micropb::MessageEncode for MotorTx {
        fn encode<IMPL_MICROPB_WRITE: ::micropb::PbWrite>(
            &self,
            encoder: &mut ::micropb::PbEncoder<IMPL_MICROPB_WRITE>,
        ) -> Result<(), IMPL_MICROPB_WRITE::Error> {
            use ::micropb::{PbVec, PbMap, PbString, FieldEncode};
            {
                let val_ref = &self.r#operation_display;
                if val_ref.0 != 0 {
                    encoder.encode_varint32(8u32)?;
                    encoder.encode_int32(val_ref.0 as _)?;
                }
            }
            {
                let val_ref = &self.r#command_buffer_full;
                if *val_ref {
                    encoder.encode_varint32(16u32)?;
                    encoder.encode_bool(*val_ref)?;
                }
            }
            {
                if let Some(val_ref) = self.r#intp_pos() {
                    encoder.encode_varint32(29u32)?;
                    encoder.encode_float(*val_ref)?;
                }
            }
            {
                if let Some(val_ref) = self.r#intp_vel() {
                    encoder.encode_varint32(37u32)?;
                    encoder.encode_float(*val_ref)?;
                }
            }
            {
                if let Some(val_ref) = self.r#intp_acc() {
                    encoder.encode_varint32(45u32)?;
                    encoder.encode_float(*val_ref)?;
                }
            }
            {
                if let Some(val_ref) = self.r#intp_jerk() {
                    encoder.encode_varint32(53u32)?;
                    encoder.encode_float(*val_ref)?;
                }
            }
            {
                if let Some(val_ref) = self.r#actual_pos() {
                    encoder.encode_varint32(61u32)?;
                    encoder.encode_float(*val_ref)?;
                }
            }
            {
                if let Some(val_ref) = self.r#actual_vel() {
                    encoder.encode_varint32(69u32)?;
                    encoder.encode_float(*val_ref)?;
                }
            }
            Ok(())
        }
        fn compute_size(&self) -> usize {
            use ::micropb::{PbVec, PbMap, PbString, FieldEncode};
            let mut size = 0;
            {
                let val_ref = &self.r#operation_display;
                if val_ref.0 != 0 {
                    size += 1usize + ::micropb::size::sizeof_int32(val_ref.0 as _);
                }
            }
            {
                let val_ref = &self.r#command_buffer_full;
                if *val_ref {
                    size += 1usize + 1;
                }
            }
            {
                if let Some(val_ref) = self.r#intp_pos() {
                    size += 1usize + 4;
                }
            }
            {
                if let Some(val_ref) = self.r#intp_vel() {
                    size += 1usize + 4;
                }
            }
            {
                if let Some(val_ref) = self.r#intp_acc() {
                    size += 1usize + 4;
                }
            }
            {
                if let Some(val_ref) = self.r#intp_jerk() {
                    size += 1usize + 4;
                }
            }
            {
                if let Some(val_ref) = self.r#actual_pos() {
                    size += 1usize + 4;
                }
            }
            {
                if let Some(val_ref) = self.r#actual_vel() {
                    size += 1usize + 4;
                }
            }
            size
        }
    }
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    #[repr(transparent)]
    pub struct Operation(pub i32);
    impl Operation {
        pub const Unspecified: Self = Self(0);
        pub const IntpPos: Self = Self(1);
        pub const IntpVel: Self = Self(2);
        pub const PidVel: Self = Self(3);
        pub const PidTune: Self = Self(4);
        pub const Stop: Self = Self(5);
    }
    impl core::default::Default for Operation {
        fn default() -> Self {
            Self(0)
        }
    }
    impl core::convert::From<i32> for Operation {
        fn from(val: i32) -> Self {
            Self(val)
        }
    }
}
pub mod sensor_ {
    pub mod Mpu6050Tx_ {
        #[derive(Debug, Default, PartialEq, Clone)]
        pub struct _Hazzer([u8; 1]);
        impl _Hazzer {
            ///Query presence of `ax`
            #[inline]
            pub fn r#ax(&self) -> bool {
                (self.0[0] & 1) != 0
            }
            ///Set presence of `ax`
            #[inline]
            pub fn set_ax(&mut self) {
                let elem = &mut self.0[0];
                *elem |= 1;
            }
            ///Clear presence of `ax`
            #[inline]
            pub fn clear_ax(&mut self) {
                let elem = &mut self.0[0];
                *elem &= !1;
            }
            ///Builder method that sets the presence of `ax`. Useful for initializing the Hazzer.
            #[inline]
            pub fn init_ax(mut self) -> Self {
                self.set_ax();
                self
            }
            ///Query presence of `ay`
            #[inline]
            pub fn r#ay(&self) -> bool {
                (self.0[0] & 2) != 0
            }
            ///Set presence of `ay`
            #[inline]
            pub fn set_ay(&mut self) {
                let elem = &mut self.0[0];
                *elem |= 2;
            }
            ///Clear presence of `ay`
            #[inline]
            pub fn clear_ay(&mut self) {
                let elem = &mut self.0[0];
                *elem &= !2;
            }
            ///Builder method that sets the presence of `ay`. Useful for initializing the Hazzer.
            #[inline]
            pub fn init_ay(mut self) -> Self {
                self.set_ay();
                self
            }
            ///Query presence of `az`
            #[inline]
            pub fn r#az(&self) -> bool {
                (self.0[0] & 4) != 0
            }
            ///Set presence of `az`
            #[inline]
            pub fn set_az(&mut self) {
                let elem = &mut self.0[0];
                *elem |= 4;
            }
            ///Clear presence of `az`
            #[inline]
            pub fn clear_az(&mut self) {
                let elem = &mut self.0[0];
                *elem &= !4;
            }
            ///Builder method that sets the presence of `az`. Useful for initializing the Hazzer.
            #[inline]
            pub fn init_az(mut self) -> Self {
                self.set_az();
                self
            }
            ///Query presence of `gx`
            #[inline]
            pub fn r#gx(&self) -> bool {
                (self.0[0] & 8) != 0
            }
            ///Set presence of `gx`
            #[inline]
            pub fn set_gx(&mut self) {
                let elem = &mut self.0[0];
                *elem |= 8;
            }
            ///Clear presence of `gx`
            #[inline]
            pub fn clear_gx(&mut self) {
                let elem = &mut self.0[0];
                *elem &= !8;
            }
            ///Builder method that sets the presence of `gx`. Useful for initializing the Hazzer.
            #[inline]
            pub fn init_gx(mut self) -> Self {
                self.set_gx();
                self
            }
            ///Query presence of `gy`
            #[inline]
            pub fn r#gy(&self) -> bool {
                (self.0[0] & 16) != 0
            }
            ///Set presence of `gy`
            #[inline]
            pub fn set_gy(&mut self) {
                let elem = &mut self.0[0];
                *elem |= 16;
            }
            ///Clear presence of `gy`
            #[inline]
            pub fn clear_gy(&mut self) {
                let elem = &mut self.0[0];
                *elem &= !16;
            }
            ///Builder method that sets the presence of `gy`. Useful for initializing the Hazzer.
            #[inline]
            pub fn init_gy(mut self) -> Self {
                self.set_gy();
                self
            }
            ///Query presence of `gz`
            #[inline]
            pub fn r#gz(&self) -> bool {
                (self.0[0] & 32) != 0
            }
            ///Set presence of `gz`
            #[inline]
            pub fn set_gz(&mut self) {
                let elem = &mut self.0[0];
                *elem |= 32;
            }
            ///Clear presence of `gz`
            #[inline]
            pub fn clear_gz(&mut self) {
                let elem = &mut self.0[0];
                *elem &= !32;
            }
            ///Builder method that sets the presence of `gz`. Useful for initializing the Hazzer.
            #[inline]
            pub fn init_gz(mut self) -> Self {
                self.set_gz();
                self
            }
        }
    }
    #[derive(Debug, PartialEq, Clone)]
    pub struct Mpu6050Tx {
        pub r#ax: f32,
        pub r#ay: f32,
        pub r#az: f32,
        pub r#gx: f32,
        pub r#gy: f32,
        pub r#gz: f32,
        pub _has: Mpu6050Tx_::_Hazzer,
    }
    impl ::core::default::Default for Mpu6050Tx {
        fn default() -> Self {
            Self {
                r#ax: ::core::default::Default::default(),
                r#ay: ::core::default::Default::default(),
                r#az: ::core::default::Default::default(),
                r#gx: ::core::default::Default::default(),
                r#gy: ::core::default::Default::default(),
                r#gz: ::core::default::Default::default(),
                _has: ::core::default::Default::default(),
            }
        }
    }
    impl Mpu6050Tx {
        ///Return a reference to `ax` as an `Option`
        #[inline]
        pub fn r#ax(&self) -> ::core::option::Option<&f32> {
            self._has.r#ax().then_some(&self.r#ax)
        }
        ///Return a mutable reference to `ax` as an `Option`
        #[inline]
        pub fn mut_ax(&mut self) -> ::core::option::Option<&mut f32> {
            self._has.r#ax().then_some(&mut self.r#ax)
        }
        ///Set the value and presence of `ax`
        #[inline]
        pub fn set_ax(&mut self, value: f32) {
            self._has.set_ax();
            self.r#ax = value.into();
        }
        ///Clear the presence of `ax`
        #[inline]
        pub fn clear_ax(&mut self) {
            self._has.clear_ax();
        }
        ///Return a reference to `ay` as an `Option`
        #[inline]
        pub fn r#ay(&self) -> ::core::option::Option<&f32> {
            self._has.r#ay().then_some(&self.r#ay)
        }
        ///Return a mutable reference to `ay` as an `Option`
        #[inline]
        pub fn mut_ay(&mut self) -> ::core::option::Option<&mut f32> {
            self._has.r#ay().then_some(&mut self.r#ay)
        }
        ///Set the value and presence of `ay`
        #[inline]
        pub fn set_ay(&mut self, value: f32) {
            self._has.set_ay();
            self.r#ay = value.into();
        }
        ///Clear the presence of `ay`
        #[inline]
        pub fn clear_ay(&mut self) {
            self._has.clear_ay();
        }
        ///Return a reference to `az` as an `Option`
        #[inline]
        pub fn r#az(&self) -> ::core::option::Option<&f32> {
            self._has.r#az().then_some(&self.r#az)
        }
        ///Return a mutable reference to `az` as an `Option`
        #[inline]
        pub fn mut_az(&mut self) -> ::core::option::Option<&mut f32> {
            self._has.r#az().then_some(&mut self.r#az)
        }
        ///Set the value and presence of `az`
        #[inline]
        pub fn set_az(&mut self, value: f32) {
            self._has.set_az();
            self.r#az = value.into();
        }
        ///Clear the presence of `az`
        #[inline]
        pub fn clear_az(&mut self) {
            self._has.clear_az();
        }
        ///Return a reference to `gx` as an `Option`
        #[inline]
        pub fn r#gx(&self) -> ::core::option::Option<&f32> {
            self._has.r#gx().then_some(&self.r#gx)
        }
        ///Return a mutable reference to `gx` as an `Option`
        #[inline]
        pub fn mut_gx(&mut self) -> ::core::option::Option<&mut f32> {
            self._has.r#gx().then_some(&mut self.r#gx)
        }
        ///Set the value and presence of `gx`
        #[inline]
        pub fn set_gx(&mut self, value: f32) {
            self._has.set_gx();
            self.r#gx = value.into();
        }
        ///Clear the presence of `gx`
        #[inline]
        pub fn clear_gx(&mut self) {
            self._has.clear_gx();
        }
        ///Return a reference to `gy` as an `Option`
        #[inline]
        pub fn r#gy(&self) -> ::core::option::Option<&f32> {
            self._has.r#gy().then_some(&self.r#gy)
        }
        ///Return a mutable reference to `gy` as an `Option`
        #[inline]
        pub fn mut_gy(&mut self) -> ::core::option::Option<&mut f32> {
            self._has.r#gy().then_some(&mut self.r#gy)
        }
        ///Set the value and presence of `gy`
        #[inline]
        pub fn set_gy(&mut self, value: f32) {
            self._has.set_gy();
            self.r#gy = value.into();
        }
        ///Clear the presence of `gy`
        #[inline]
        pub fn clear_gy(&mut self) {
            self._has.clear_gy();
        }
        ///Return a reference to `gz` as an `Option`
        #[inline]
        pub fn r#gz(&self) -> ::core::option::Option<&f32> {
            self._has.r#gz().then_some(&self.r#gz)
        }
        ///Return a mutable reference to `gz` as an `Option`
        #[inline]
        pub fn mut_gz(&mut self) -> ::core::option::Option<&mut f32> {
            self._has.r#gz().then_some(&mut self.r#gz)
        }
        ///Set the value and presence of `gz`
        #[inline]
        pub fn set_gz(&mut self, value: f32) {
            self._has.set_gz();
            self.r#gz = value.into();
        }
        ///Clear the presence of `gz`
        #[inline]
        pub fn clear_gz(&mut self) {
            self._has.clear_gz();
        }
    }
    impl ::micropb::MessageDecode for Mpu6050Tx {
        fn decode<IMPL_MICROPB_READ: ::micropb::PbRead>(
            &mut self,
            decoder: &mut ::micropb::PbDecoder<IMPL_MICROPB_READ>,
            len: usize,
        ) -> Result<(), ::micropb::DecodeError<IMPL_MICROPB_READ::Error>> {
            use ::micropb::{PbVec, PbMap, PbString, FieldDecode};
            let before = decoder.bytes_read();
            while decoder.bytes_read() - before < len {
                let tag = decoder.decode_tag()?;
                match tag.field_num() {
                    0 => return Err(::micropb::DecodeError::ZeroField),
                    1u32 => {
                        let mut_ref = &mut self.r#ax;
                        {
                            let val = decoder.decode_float()?;
                            *mut_ref = val as _;
                        };
                        self._has.set_ax();
                    }
                    2u32 => {
                        let mut_ref = &mut self.r#ay;
                        {
                            let val = decoder.decode_float()?;
                            *mut_ref = val as _;
                        };
                        self._has.set_ay();
                    }
                    3u32 => {
                        let mut_ref = &mut self.r#az;
                        {
                            let val = decoder.decode_float()?;
                            *mut_ref = val as _;
                        };
                        self._has.set_az();
                    }
                    4u32 => {
                        let mut_ref = &mut self.r#gx;
                        {
                            let val = decoder.decode_float()?;
                            *mut_ref = val as _;
                        };
                        self._has.set_gx();
                    }
                    5u32 => {
                        let mut_ref = &mut self.r#gy;
                        {
                            let val = decoder.decode_float()?;
                            *mut_ref = val as _;
                        };
                        self._has.set_gy();
                    }
                    6u32 => {
                        let mut_ref = &mut self.r#gz;
                        {
                            let val = decoder.decode_float()?;
                            *mut_ref = val as _;
                        };
                        self._has.set_gz();
                    }
                    _ => {
                        decoder.skip_wire_value(tag.wire_type())?;
                    }
                }
            }
            Ok(())
        }
    }
    impl ::micropb::MessageEncode for Mpu6050Tx {
        fn encode<IMPL_MICROPB_WRITE: ::micropb::PbWrite>(
            &self,
            encoder: &mut ::micropb::PbEncoder<IMPL_MICROPB_WRITE>,
        ) -> Result<(), IMPL_MICROPB_WRITE::Error> {
            use ::micropb::{PbVec, PbMap, PbString, FieldEncode};
            {
                if let Some(val_ref) = self.r#ax() {
                    encoder.encode_varint32(13u32)?;
                    encoder.encode_float(*val_ref)?;
                }
            }
            {
                if let Some(val_ref) = self.r#ay() {
                    encoder.encode_varint32(21u32)?;
                    encoder.encode_float(*val_ref)?;
                }
            }
            {
                if let Some(val_ref) = self.r#az() {
                    encoder.encode_varint32(29u32)?;
                    encoder.encode_float(*val_ref)?;
                }
            }
            {
                if let Some(val_ref) = self.r#gx() {
                    encoder.encode_varint32(37u32)?;
                    encoder.encode_float(*val_ref)?;
                }
            }
            {
                if let Some(val_ref) = self.r#gy() {
                    encoder.encode_varint32(45u32)?;
                    encoder.encode_float(*val_ref)?;
                }
            }
            {
                if let Some(val_ref) = self.r#gz() {
                    encoder.encode_varint32(53u32)?;
                    encoder.encode_float(*val_ref)?;
                }
            }
            Ok(())
        }
        fn compute_size(&self) -> usize {
            use ::micropb::{PbVec, PbMap, PbString, FieldEncode};
            let mut size = 0;
            {
                if let Some(val_ref) = self.r#ax() {
                    size += 1usize + 4;
                }
            }
            {
                if let Some(val_ref) = self.r#ay() {
                    size += 1usize + 4;
                }
            }
            {
                if let Some(val_ref) = self.r#az() {
                    size += 1usize + 4;
                }
            }
            {
                if let Some(val_ref) = self.r#gx() {
                    size += 1usize + 4;
                }
            }
            {
                if let Some(val_ref) = self.r#gy() {
                    size += 1usize + 4;
                }
            }
            {
                if let Some(val_ref) = self.r#gz() {
                    size += 1usize + 4;
                }
            }
            size
        }
    }
}
pub mod command_ {
    pub mod CommandRx_ {
        #[derive(Debug, Default, PartialEq, Clone)]
        pub struct _Hazzer([u8; 1]);
        impl _Hazzer {
            ///Query presence of `left_motor`
            #[inline]
            pub fn r#left_motor(&self) -> bool {
                (self.0[0] & 1) != 0
            }
            ///Set presence of `left_motor`
            #[inline]
            pub fn set_left_motor(&mut self) {
                let elem = &mut self.0[0];
                *elem |= 1;
            }
            ///Clear presence of `left_motor`
            #[inline]
            pub fn clear_left_motor(&mut self) {
                let elem = &mut self.0[0];
                *elem &= !1;
            }
            ///Builder method that sets the presence of `left_motor`. Useful for initializing the Hazzer.
            #[inline]
            pub fn init_left_motor(mut self) -> Self {
                self.set_left_motor();
                self
            }
            ///Query presence of `right_motor`
            #[inline]
            pub fn r#right_motor(&self) -> bool {
                (self.0[0] & 2) != 0
            }
            ///Set presence of `right_motor`
            #[inline]
            pub fn set_right_motor(&mut self) {
                let elem = &mut self.0[0];
                *elem |= 2;
            }
            ///Clear presence of `right_motor`
            #[inline]
            pub fn clear_right_motor(&mut self) {
                let elem = &mut self.0[0];
                *elem &= !2;
            }
            ///Builder method that sets the presence of `right_motor`. Useful for initializing the Hazzer.
            #[inline]
            pub fn init_right_motor(mut self) -> Self {
                self.set_right_motor();
                self
            }
        }
    }
    #[derive(Debug, PartialEq, Clone)]
    pub struct CommandRx {
        pub r#left_motor: super::motor_::MotorRx,
        pub r#right_motor: super::motor_::MotorRx,
        pub _has: CommandRx_::_Hazzer,
    }
    impl ::core::default::Default for CommandRx {
        fn default() -> Self {
            Self {
                r#left_motor: ::core::default::Default::default(),
                r#right_motor: ::core::default::Default::default(),
                _has: ::core::default::Default::default(),
            }
        }
    }
    impl CommandRx {
        ///Return a reference to `left_motor` as an `Option`
        #[inline]
        pub fn r#left_motor(&self) -> ::core::option::Option<&super::motor_::MotorRx> {
            self._has.r#left_motor().then_some(&self.r#left_motor)
        }
        ///Return a mutable reference to `left_motor` as an `Option`
        #[inline]
        pub fn mut_left_motor(
            &mut self,
        ) -> ::core::option::Option<&mut super::motor_::MotorRx> {
            self._has.r#left_motor().then_some(&mut self.r#left_motor)
        }
        ///Set the value and presence of `left_motor`
        #[inline]
        pub fn set_left_motor(&mut self, value: super::motor_::MotorRx) {
            self._has.set_left_motor();
            self.r#left_motor = value.into();
        }
        ///Clear the presence of `left_motor`
        #[inline]
        pub fn clear_left_motor(&mut self) {
            self._has.clear_left_motor();
        }
        ///Return a reference to `right_motor` as an `Option`
        #[inline]
        pub fn r#right_motor(&self) -> ::core::option::Option<&super::motor_::MotorRx> {
            self._has.r#right_motor().then_some(&self.r#right_motor)
        }
        ///Return a mutable reference to `right_motor` as an `Option`
        #[inline]
        pub fn mut_right_motor(
            &mut self,
        ) -> ::core::option::Option<&mut super::motor_::MotorRx> {
            self._has.r#right_motor().then_some(&mut self.r#right_motor)
        }
        ///Set the value and presence of `right_motor`
        #[inline]
        pub fn set_right_motor(&mut self, value: super::motor_::MotorRx) {
            self._has.set_right_motor();
            self.r#right_motor = value.into();
        }
        ///Clear the presence of `right_motor`
        #[inline]
        pub fn clear_right_motor(&mut self) {
            self._has.clear_right_motor();
        }
    }
    impl ::micropb::MessageDecode for CommandRx {
        fn decode<IMPL_MICROPB_READ: ::micropb::PbRead>(
            &mut self,
            decoder: &mut ::micropb::PbDecoder<IMPL_MICROPB_READ>,
            len: usize,
        ) -> Result<(), ::micropb::DecodeError<IMPL_MICROPB_READ::Error>> {
            use ::micropb::{PbVec, PbMap, PbString, FieldDecode};
            let before = decoder.bytes_read();
            while decoder.bytes_read() - before < len {
                let tag = decoder.decode_tag()?;
                match tag.field_num() {
                    0 => return Err(::micropb::DecodeError::ZeroField),
                    1u32 => {
                        let mut_ref = &mut self.r#left_motor;
                        {
                            mut_ref.decode_len_delimited(decoder)?;
                        };
                        self._has.set_left_motor();
                    }
                    2u32 => {
                        let mut_ref = &mut self.r#right_motor;
                        {
                            mut_ref.decode_len_delimited(decoder)?;
                        };
                        self._has.set_right_motor();
                    }
                    _ => {
                        decoder.skip_wire_value(tag.wire_type())?;
                    }
                }
            }
            Ok(())
        }
    }
    impl ::micropb::MessageEncode for CommandRx {
        fn encode<IMPL_MICROPB_WRITE: ::micropb::PbWrite>(
            &self,
            encoder: &mut ::micropb::PbEncoder<IMPL_MICROPB_WRITE>,
        ) -> Result<(), IMPL_MICROPB_WRITE::Error> {
            use ::micropb::{PbVec, PbMap, PbString, FieldEncode};
            {
                if let Some(val_ref) = self.r#left_motor() {
                    encoder.encode_varint32(10u32)?;
                    val_ref.encode_len_delimited(encoder)?;
                }
            }
            {
                if let Some(val_ref) = self.r#right_motor() {
                    encoder.encode_varint32(18u32)?;
                    val_ref.encode_len_delimited(encoder)?;
                }
            }
            Ok(())
        }
        fn compute_size(&self) -> usize {
            use ::micropb::{PbVec, PbMap, PbString, FieldEncode};
            let mut size = 0;
            {
                if let Some(val_ref) = self.r#left_motor() {
                    size
                        += 1usize
                            + ::micropb::size::sizeof_len_record(val_ref.compute_size());
                }
            }
            {
                if let Some(val_ref) = self.r#right_motor() {
                    size
                        += 1usize
                            + ::micropb::size::sizeof_len_record(val_ref.compute_size());
                }
            }
            size
        }
    }
    pub mod CommandTx_ {
        #[derive(Debug, Default, PartialEq, Clone)]
        pub struct _Hazzer([u8; 1]);
        impl _Hazzer {
            ///Query presence of `left_motor`
            #[inline]
            pub fn r#left_motor(&self) -> bool {
                (self.0[0] & 1) != 0
            }
            ///Set presence of `left_motor`
            #[inline]
            pub fn set_left_motor(&mut self) {
                let elem = &mut self.0[0];
                *elem |= 1;
            }
            ///Clear presence of `left_motor`
            #[inline]
            pub fn clear_left_motor(&mut self) {
                let elem = &mut self.0[0];
                *elem &= !1;
            }
            ///Builder method that sets the presence of `left_motor`. Useful for initializing the Hazzer.
            #[inline]
            pub fn init_left_motor(mut self) -> Self {
                self.set_left_motor();
                self
            }
            ///Query presence of `right_motor`
            #[inline]
            pub fn r#right_motor(&self) -> bool {
                (self.0[0] & 2) != 0
            }
            ///Set presence of `right_motor`
            #[inline]
            pub fn set_right_motor(&mut self) {
                let elem = &mut self.0[0];
                *elem |= 2;
            }
            ///Clear presence of `right_motor`
            #[inline]
            pub fn clear_right_motor(&mut self) {
                let elem = &mut self.0[0];
                *elem &= !2;
            }
            ///Builder method that sets the presence of `right_motor`. Useful for initializing the Hazzer.
            #[inline]
            pub fn init_right_motor(mut self) -> Self {
                self.set_right_motor();
                self
            }
            ///Query presence of `mpu6050`
            #[inline]
            pub fn r#mpu6050(&self) -> bool {
                (self.0[0] & 4) != 0
            }
            ///Set presence of `mpu6050`
            #[inline]
            pub fn set_mpu6050(&mut self) {
                let elem = &mut self.0[0];
                *elem |= 4;
            }
            ///Clear presence of `mpu6050`
            #[inline]
            pub fn clear_mpu6050(&mut self) {
                let elem = &mut self.0[0];
                *elem &= !4;
            }
            ///Builder method that sets the presence of `mpu6050`. Useful for initializing the Hazzer.
            #[inline]
            pub fn init_mpu6050(mut self) -> Self {
                self.set_mpu6050();
                self
            }
        }
    }
    #[derive(Debug, PartialEq, Clone)]
    pub struct CommandTx {
        pub r#left_motor: super::motor_::MotorTx,
        pub r#right_motor: super::motor_::MotorTx,
        pub r#mpu6050: super::sensor_::Mpu6050Tx,
        pub _has: CommandTx_::_Hazzer,
    }
    impl ::core::default::Default for CommandTx {
        fn default() -> Self {
            Self {
                r#left_motor: ::core::default::Default::default(),
                r#right_motor: ::core::default::Default::default(),
                r#mpu6050: ::core::default::Default::default(),
                _has: ::core::default::Default::default(),
            }
        }
    }
    impl CommandTx {
        ///Return a reference to `left_motor` as an `Option`
        #[inline]
        pub fn r#left_motor(&self) -> ::core::option::Option<&super::motor_::MotorTx> {
            self._has.r#left_motor().then_some(&self.r#left_motor)
        }
        ///Return a mutable reference to `left_motor` as an `Option`
        #[inline]
        pub fn mut_left_motor(
            &mut self,
        ) -> ::core::option::Option<&mut super::motor_::MotorTx> {
            self._has.r#left_motor().then_some(&mut self.r#left_motor)
        }
        ///Set the value and presence of `left_motor`
        #[inline]
        pub fn set_left_motor(&mut self, value: super::motor_::MotorTx) {
            self._has.set_left_motor();
            self.r#left_motor = value.into();
        }
        ///Clear the presence of `left_motor`
        #[inline]
        pub fn clear_left_motor(&mut self) {
            self._has.clear_left_motor();
        }
        ///Return a reference to `right_motor` as an `Option`
        #[inline]
        pub fn r#right_motor(&self) -> ::core::option::Option<&super::motor_::MotorTx> {
            self._has.r#right_motor().then_some(&self.r#right_motor)
        }
        ///Return a mutable reference to `right_motor` as an `Option`
        #[inline]
        pub fn mut_right_motor(
            &mut self,
        ) -> ::core::option::Option<&mut super::motor_::MotorTx> {
            self._has.r#right_motor().then_some(&mut self.r#right_motor)
        }
        ///Set the value and presence of `right_motor`
        #[inline]
        pub fn set_right_motor(&mut self, value: super::motor_::MotorTx) {
            self._has.set_right_motor();
            self.r#right_motor = value.into();
        }
        ///Clear the presence of `right_motor`
        #[inline]
        pub fn clear_right_motor(&mut self) {
            self._has.clear_right_motor();
        }
        ///Return a reference to `mpu6050` as an `Option`
        #[inline]
        pub fn r#mpu6050(&self) -> ::core::option::Option<&super::sensor_::Mpu6050Tx> {
            self._has.r#mpu6050().then_some(&self.r#mpu6050)
        }
        ///Return a mutable reference to `mpu6050` as an `Option`
        #[inline]
        pub fn mut_mpu6050(
            &mut self,
        ) -> ::core::option::Option<&mut super::sensor_::Mpu6050Tx> {
            self._has.r#mpu6050().then_some(&mut self.r#mpu6050)
        }
        ///Set the value and presence of `mpu6050`
        #[inline]
        pub fn set_mpu6050(&mut self, value: super::sensor_::Mpu6050Tx) {
            self._has.set_mpu6050();
            self.r#mpu6050 = value.into();
        }
        ///Clear the presence of `mpu6050`
        #[inline]
        pub fn clear_mpu6050(&mut self) {
            self._has.clear_mpu6050();
        }
    }
    impl ::micropb::MessageDecode for CommandTx {
        fn decode<IMPL_MICROPB_READ: ::micropb::PbRead>(
            &mut self,
            decoder: &mut ::micropb::PbDecoder<IMPL_MICROPB_READ>,
            len: usize,
        ) -> Result<(), ::micropb::DecodeError<IMPL_MICROPB_READ::Error>> {
            use ::micropb::{PbVec, PbMap, PbString, FieldDecode};
            let before = decoder.bytes_read();
            while decoder.bytes_read() - before < len {
                let tag = decoder.decode_tag()?;
                match tag.field_num() {
                    0 => return Err(::micropb::DecodeError::ZeroField),
                    1u32 => {
                        let mut_ref = &mut self.r#left_motor;
                        {
                            mut_ref.decode_len_delimited(decoder)?;
                        };
                        self._has.set_left_motor();
                    }
                    2u32 => {
                        let mut_ref = &mut self.r#right_motor;
                        {
                            mut_ref.decode_len_delimited(decoder)?;
                        };
                        self._has.set_right_motor();
                    }
                    3u32 => {
                        let mut_ref = &mut self.r#mpu6050;
                        {
                            mut_ref.decode_len_delimited(decoder)?;
                        };
                        self._has.set_mpu6050();
                    }
                    _ => {
                        decoder.skip_wire_value(tag.wire_type())?;
                    }
                }
            }
            Ok(())
        }
    }
    impl ::micropb::MessageEncode for CommandTx {
        fn encode<IMPL_MICROPB_WRITE: ::micropb::PbWrite>(
            &self,
            encoder: &mut ::micropb::PbEncoder<IMPL_MICROPB_WRITE>,
        ) -> Result<(), IMPL_MICROPB_WRITE::Error> {
            use ::micropb::{PbVec, PbMap, PbString, FieldEncode};
            {
                if let Some(val_ref) = self.r#left_motor() {
                    encoder.encode_varint32(10u32)?;
                    val_ref.encode_len_delimited(encoder)?;
                }
            }
            {
                if let Some(val_ref) = self.r#right_motor() {
                    encoder.encode_varint32(18u32)?;
                    val_ref.encode_len_delimited(encoder)?;
                }
            }
            {
                if let Some(val_ref) = self.r#mpu6050() {
                    encoder.encode_varint32(26u32)?;
                    val_ref.encode_len_delimited(encoder)?;
                }
            }
            Ok(())
        }
        fn compute_size(&self) -> usize {
            use ::micropb::{PbVec, PbMap, PbString, FieldEncode};
            let mut size = 0;
            {
                if let Some(val_ref) = self.r#left_motor() {
                    size
                        += 1usize
                            + ::micropb::size::sizeof_len_record(val_ref.compute_size());
                }
            }
            {
                if let Some(val_ref) = self.r#right_motor() {
                    size
                        += 1usize
                            + ::micropb::size::sizeof_len_record(val_ref.compute_size());
                }
            }
            {
                if let Some(val_ref) = self.r#mpu6050() {
                    size
                        += 1usize
                            + ::micropb::size::sizeof_len_record(val_ref.compute_size());
                }
            }
            size
        }
    }
}
