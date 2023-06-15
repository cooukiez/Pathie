#[macro_export]
macro_rules! bitset {
    ($num : expr, $bit : expr) => {
        // Shift 1 to right pos
        // 'or' operator => If one bit of
        // the two is 1 => 1 returned

        $num | 1 << $bit
    };
}

#[macro_export]
macro_rules! bitclear {
    ($num : expr, $bit : expr) => {
        // Shift 1 to right pos
        // Invert all and then do 'and' operator
        // 'and' is zero when both are not 1
        // therefore if you have only 1 => 
        // number will not be changed

        $num & !(1 << $bit)
    };
}

#[macro_export]
macro_rules! bitflip {
    ($num : expr, $bit : expr) => {
        $num ^ 1 << $bit
    };
}

#[macro_export]
macro_rules! bitcheck {
    ($num : expr, $bit : expr) => {
        // Shift the number, so that
        // the last bit is equal to the bit
        // that is needed to be read
        // Check if last bit is 1 with 'and' operator

        ($num >> $bit) & 1
    };
}

#[macro_export]
macro_rules! set_bit {
    ($value : expr, $bit : expr, $bit_value : expr) => {{
        // When true set bit to 1
        // Operation to set bit is explained
        // above, same for clearing

        if $bit_value == true {
            $value | (1 << $bit)
        } else {
            $value & !(1 << $bit)
        }
    }};
}

#[macro_export]
macro_rules! read_bitrange {
    ($num : expr, $s : expr, $e : expr) => {{
        // Create mask of the range
        // For Example
        // mask = 0000 0000 1111 1111 0000 0000 0000 0000

        let mask = (!0u32 >> $e) << ($s - 1);

        // Compare num to mask, at each 0 in the mask the
        // value will be 0 because of 'and' operator
        // At each 1 the value will be equal to the value in num

        ($num & mask) >> ($s - 1)
    }};
}

#[macro_export]
macro_rules! write_bitrange {
    ($num : expr, $rep : expr, $s : expr, $e : expr) => {{
        // Create mask of the range
        // For Example
        // mask = 0000 0000 1111 1111 0000 0000 0000 0000

        let mask = (!0u32 >> $e) << ($s - 1);

        // Get Invert mask and use 'and' operator to set
        // the all 1 from the mask in the num to 0 and
        // leave the rest be, after that copare the
        // rep to the mask to copy the rep into the range and
        // after that insert with 'or' operator

        ($num & !(mask)) | ($rep & mask)
    }};
}