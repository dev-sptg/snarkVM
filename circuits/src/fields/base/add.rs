// Copyright (C) 2019-2022 Aleo Systems Inc.
// This file is part of the snarkVM library.

// The snarkVM library is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// The snarkVM library is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with the snarkVM library. If not, see <https://www.gnu.org/licenses/>.

use super::*;

impl<E: Environment> Add<BaseField<E>> for BaseField<E> {
    type Output = BaseField<E>;

    fn add(self, other: BaseField<E>) -> Self::Output {
        self + &other
    }
}

impl<E: Environment> Add<BaseField<E>> for &BaseField<E> {
    type Output = BaseField<E>;

    fn add(self, other: BaseField<E>) -> Self::Output {
        self + &other
    }
}

impl<E: Environment> Add<&BaseField<E>> for BaseField<E> {
    type Output = BaseField<E>;

    fn add(self, other: &BaseField<E>) -> Self::Output {
        BaseField(self.0 + &other.0)
    }
}

impl<E: Environment> Add<&BaseField<E>> for &BaseField<E> {
    type Output = BaseField<E>;

    fn add(self, other: &BaseField<E>) -> Self::Output {
        (*self).clone() + other
    }
}

impl<E: Environment> AddAssign<BaseField<E>> for BaseField<E> {
    fn add_assign(&mut self, other: BaseField<E>) {
        *self += &other;
    }
}

impl<E: Environment> AddAssign<&BaseField<E>> for BaseField<E> {
    fn add_assign(&mut self, other: &BaseField<E>) {
        self.0 += &other.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Circuit;
    use snarkvm_utilities::UniformRand;

    use rand::thread_rng;

    const ITERATIONS: usize = 100_000;

    fn check_add(
        name: &str,
        expected: &<Circuit as Environment>::BaseField,
        a: &BaseField<Circuit>,
        b: &BaseField<Circuit>,
        num_constants: usize,
        num_public: usize,
        num_private: usize,
        num_constraints: usize,
    ) {
        Circuit::scoped(name, |scope| {
            let candidate = a + b;
            assert_eq!(
                *expected,
                candidate.eject_value(),
                "{} != {} := ({} + {})",
                expected,
                candidate.eject_value(),
                a.eject_value(),
                b.eject_value()
            );

            assert_eq!(num_constants, scope.num_constants_in_scope());
            assert_eq!(num_public, scope.num_public_in_scope());
            assert_eq!(num_private, scope.num_private_in_scope());
            assert_eq!(num_constraints, scope.num_constraints_in_scope());
            assert!(Circuit::is_satisfied());
        });
    }

    fn check_add_assign(
        name: &str,
        expected: &<Circuit as Environment>::BaseField,
        a: &BaseField<Circuit>,
        b: &BaseField<Circuit>,
        num_constants: usize,
        num_public: usize,
        num_private: usize,
        num_constraints: usize,
    ) {
        Circuit::scoped(name, |scope| {
            let mut candidate = a.clone();
            candidate += b;
            assert_eq!(
                *expected,
                candidate.eject_value(),
                "{} != {} := ({} + {})",
                expected,
                candidate.eject_value(),
                a.eject_value(),
                b.eject_value()
            );

            assert_eq!(num_constants, scope.num_constants_in_scope());
            assert_eq!(num_public, scope.num_public_in_scope());
            assert_eq!(num_private, scope.num_private_in_scope());
            assert_eq!(num_constraints, scope.num_constraints_in_scope());
            assert!(Circuit::is_satisfied());
        });
    }

    #[test]
    fn test_constant_plus_constant() {
        for i in 0..ITERATIONS {
            let first = UniformRand::rand(&mut thread_rng());
            let second = UniformRand::rand(&mut thread_rng());

            let expected = first + second;
            let a = BaseField::<Circuit>::new(Mode::Constant, first);
            let b = BaseField::<Circuit>::new(Mode::Constant, second);

            let name = format!("Add: a + b {}", i);
            check_add(&name, &expected, &a, &b, 0, 0, 0, 0);
            let name = format!("AddAssign: a + b {}", i);
            check_add_assign(&name, &expected, &a, &b, 0, 0, 0, 0);
        }
    }

    #[test]
    fn test_constant_plus_public() {
        for i in 0..ITERATIONS {
            let first = UniformRand::rand(&mut thread_rng());
            let second = UniformRand::rand(&mut thread_rng());

            let expected = first + second;
            let a = BaseField::<Circuit>::new(Mode::Constant, first);
            let b = BaseField::<Circuit>::new(Mode::Public, second);

            let name = format!("Add: a + b {}", i);
            check_add(&name, &expected, &a, &b, 0, 0, 0, 0);
            let name = format!("AddAssign: a + b {}", i);
            check_add_assign(&name, &expected, &a, &b, 0, 0, 0, 0);
        }
    }

    #[test]
    fn test_public_plus_constant() {
        for i in 0..ITERATIONS {
            let first = UniformRand::rand(&mut thread_rng());
            let second = UniformRand::rand(&mut thread_rng());

            let expected = first + second;
            let a = BaseField::<Circuit>::new(Mode::Public, first);
            let b = BaseField::<Circuit>::new(Mode::Constant, second);

            let name = format!("Add: a + b {}", i);
            check_add(&name, &expected, &a, &b, 0, 0, 0, 0);
            let name = format!("AddAssign: a + b {}", i);
            check_add_assign(&name, &expected, &a, &b, 0, 0, 0, 0);
        }
    }

    #[test]
    fn test_constant_plus_private() {
        for i in 0..ITERATIONS {
            let first = UniformRand::rand(&mut thread_rng());
            let second = UniformRand::rand(&mut thread_rng());

            let expected = first + second;
            let a = BaseField::<Circuit>::new(Mode::Constant, first);
            let b = BaseField::<Circuit>::new(Mode::Private, second);

            let name = format!("Add: a + b {}", i);
            check_add(&name, &expected, &a, &b, 0, 0, 0, 0);
            let name = format!("AddAssign: a + b {}", i);
            check_add_assign(&name, &expected, &a, &b, 0, 0, 0, 0);
        }
    }

    #[test]
    fn test_private_plus_constant() {
        for i in 0..ITERATIONS {
            let first = UniformRand::rand(&mut thread_rng());
            let second = UniformRand::rand(&mut thread_rng());

            let expected = first + second;
            let a = BaseField::<Circuit>::new(Mode::Private, first);
            let b = BaseField::<Circuit>::new(Mode::Constant, second);

            let name = format!("Add: a + b {}", i);
            check_add(&name, &expected, &a, &b, 0, 0, 0, 0);
            let name = format!("AddAssign: a + b {}", i);
            check_add_assign(&name, &expected, &a, &b, 0, 0, 0, 0);
        }
    }

    #[test]
    fn test_public_plus_public() {
        for i in 0..ITERATIONS {
            let first = UniformRand::rand(&mut thread_rng());
            let second = UniformRand::rand(&mut thread_rng());

            let expected = first + second;
            let a = BaseField::<Circuit>::new(Mode::Public, first);
            let b = BaseField::<Circuit>::new(Mode::Public, second);

            let name = format!("Add: a + b {}", i);
            check_add(&name, &expected, &a, &b, 0, 0, 0, 0);
            let name = format!("AddAssign: a + b {}", i);
            check_add_assign(&name, &expected, &a, &b, 0, 0, 0, 0);
        }
    }

    #[test]
    fn test_public_plus_private() {
        for i in 0..ITERATIONS {
            let first = UniformRand::rand(&mut thread_rng());
            let second = UniformRand::rand(&mut thread_rng());

            let expected = first + second;
            let a = BaseField::<Circuit>::new(Mode::Public, first);
            let b = BaseField::<Circuit>::new(Mode::Private, second);

            let name = format!("Add: a + b {}", i);
            check_add(&name, &expected, &a, &b, 0, 0, 0, 0);
            let name = format!("AddAssign: a + b {}", i);
            check_add_assign(&name, &expected, &a, &b, 0, 0, 0, 0);
        }
    }

    #[test]
    fn test_private_plus_public() {
        for i in 0..ITERATIONS {
            let first = UniformRand::rand(&mut thread_rng());
            let second = UniformRand::rand(&mut thread_rng());

            let expected = first + second;
            let a = BaseField::<Circuit>::new(Mode::Private, first);
            let b = BaseField::<Circuit>::new(Mode::Public, second);

            let name = format!("Add: a + b {}", i);
            check_add(&name, &expected, &a, &b, 0, 0, 0, 0);
            let name = format!("AddAssign: a + b {}", i);
            check_add_assign(&name, &expected, &a, &b, 0, 0, 0, 0);
        }
    }

    #[test]
    fn test_private_plus_private() {
        for i in 0..ITERATIONS {
            let first = UniformRand::rand(&mut thread_rng());
            let second = UniformRand::rand(&mut thread_rng());

            let expected = first + second;
            let a = BaseField::<Circuit>::new(Mode::Private, first);
            let b = BaseField::<Circuit>::new(Mode::Private, second);

            let name = format!("Add: a + b {}", i);
            check_add(&name, &expected, &a, &b, 0, 0, 0, 0);
            let name = format!("AddAssign: a + b {}", i);
            check_add_assign(&name, &expected, &a, &b, 0, 0, 0, 0);
        }
    }

    #[test]
    fn test_0_plus_0() {
        let zero = <Circuit as Environment>::BaseField::zero();

        let candidate = BaseField::<Circuit>::zero() + BaseField::zero();
        assert_eq!(zero, candidate.eject_value());

        let candidate = BaseField::<Circuit>::zero() + &BaseField::zero();
        assert_eq!(zero, candidate.eject_value());

        let candidate = BaseField::<Circuit>::new(Mode::Public, zero) + BaseField::new(Mode::Public, zero);
        assert_eq!(zero, candidate.eject_value());

        let candidate = BaseField::<Circuit>::new(Mode::Public, zero) + BaseField::new(Mode::Private, zero);
        assert_eq!(zero, candidate.eject_value());

        let candidate = BaseField::<Circuit>::new(Mode::Private, zero) + BaseField::new(Mode::Private, zero);
        assert_eq!(zero, candidate.eject_value());
    }

    #[test]
    fn test_0_plus_1() {
        let zero = <Circuit as Environment>::BaseField::zero();
        let one = <Circuit as Environment>::BaseField::one();

        let candidate = BaseField::<Circuit>::zero() + BaseField::one();
        assert_eq!(one, candidate.eject_value());

        let candidate = BaseField::<Circuit>::zero() + &BaseField::one();
        assert_eq!(one, candidate.eject_value());

        let candidate = BaseField::<Circuit>::one() + BaseField::zero();
        assert_eq!(one, candidate.eject_value());

        let candidate = BaseField::<Circuit>::one() + &BaseField::zero();
        assert_eq!(one, candidate.eject_value());

        let candidate = BaseField::<Circuit>::new(Mode::Public, one) + BaseField::new(Mode::Public, zero);
        assert_eq!(one, candidate.eject_value());

        let candidate = BaseField::<Circuit>::new(Mode::Public, one) + BaseField::new(Mode::Private, zero);
        assert_eq!(one, candidate.eject_value());

        let candidate = BaseField::<Circuit>::new(Mode::Private, one) + BaseField::new(Mode::Private, zero);
        assert_eq!(one, candidate.eject_value());
    }

    #[test]
    fn test_1_plus_1() {
        let one = <Circuit as Environment>::BaseField::one();
        let two = one + one;

        let candidate = BaseField::<Circuit>::one() + BaseField::one();
        assert_eq!(two, candidate.eject_value());

        let candidate = BaseField::<Circuit>::one() + &BaseField::one();
        assert_eq!(two, candidate.eject_value());

        let candidate = BaseField::<Circuit>::new(Mode::Public, one) + BaseField::new(Mode::Public, one);
        assert_eq!(two, candidate.eject_value());

        let candidate = BaseField::<Circuit>::new(Mode::Private, one) + BaseField::new(Mode::Public, one);
        assert_eq!(two, candidate.eject_value());

        let candidate = BaseField::<Circuit>::new(Mode::Private, one) + BaseField::new(Mode::Private, one);
        assert_eq!(two, candidate.eject_value());
    }

    #[test]
    fn test_1_plus_2() {
        let one = <Circuit as Environment>::BaseField::one();
        let two = one + one;
        let three = two + one;

        let candidate_two = BaseField::<Circuit>::one() + BaseField::one();
        let candidate = candidate_two + BaseField::one();
        assert_eq!(three, candidate.eject_value());

        let candidate_two = BaseField::<Circuit>::one() + &BaseField::one();
        let candidate = candidate_two + &BaseField::one();
        assert_eq!(three, candidate.eject_value());

        let candidate = BaseField::<Circuit>::new(Mode::Public, one) + BaseField::new(Mode::Public, two);
        assert_eq!(three, candidate.eject_value());

        let candidate = BaseField::<Circuit>::new(Mode::Private, one) + BaseField::new(Mode::Public, two);
        assert_eq!(three, candidate.eject_value());

        let candidate = BaseField::<Circuit>::new(Mode::Private, one) + BaseField::new(Mode::Private, two);
        assert_eq!(three, candidate.eject_value());
    }
}
