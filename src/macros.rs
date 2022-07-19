/// Creates a enum list of combined race identifiers.
#[macro_export] macro_rules! define_race_enum {
    (
	    pub enum $name:ident {
	        $(
	            $([$id:expr]($race:ident, $gender:ident $(, $subrace:ident)?))*
	        ),+$(,)?
	    }
    ) => {
        paste! {
            #[derive(PartialEq, Debug)]

            pub enum $name {
                $(
                    $([<$race $($subrace)? $gender>] = $id)*
                    ,
                )+
            }
        }

        paste! {
            pub fn convert_to_internal(race : Race, subrace : Subrace, gender : Gender) -> Option<$name> {
                $(
                    $(if race == $race $(&& subrace == $subrace)? && gender == $gender {
                        return Some($name::[<$race $($subrace)? $gender>])
                    })*
                )+

                None
            }
        }
    };
}
