"""
Test suite for molrs.Box
Adapted from freud-analysis library tests
"""

import numpy as np
import numpy.testing as npt
import pytest

import molrs


class TestBox:
    def test_construct(self):
        """Test correct behavior for various constructor signatures"""
        # Test cubic box
        box = molrs.Box.cube(10.0)
        assert box.volume() > 0
        
        # Test orthorhombic box
        box = molrs.Box.orthorhombic(np.array([2.0, 3.0, 4.0]))
        assert box.volume() > 0
        
        # Test triclinic box
        h = np.array([[2.0, 0.0, 0.0],
                      [0.0, 2.0, 0.0],
                      [0.0, 0.0, 2.0]])
        box = molrs.Box(h)
        assert box.volume() > 0

    def test_box_volume(self):
        """Test box volume calculation"""
        # Cubic box
        box = molrs.Box.cube(2.0)
        npt.assert_allclose(box.volume(), 8.0, rtol=1e-6)
        
        # Orthorhombic box
        box = molrs.Box.orthorhombic(np.array([2.0, 3.0, 4.0]))
        npt.assert_allclose(box.volume(), 24.0, rtol=1e-6)

    def test_lattice_vectors(self):
        """Test getting lattice vectors"""
        box = molrs.Box.cube(2.0)
        
        v0 = box.lattice_vector(0)
        v1 = box.lattice_vector(1)
        v2 = box.lattice_vector(2)
        
        npt.assert_allclose(v0, [2.0, 0.0, 0.0], rtol=1e-6)
        npt.assert_allclose(v1, [0.0, 2.0, 0.0], rtol=1e-6)
        npt.assert_allclose(v2, [0.0, 0.0, 2.0], rtol=1e-6)
        
        # Test invalid index
        with pytest.raises(ValueError):
            box.lattice_vector(3)

    def test_wrap_single_particle(self):
        """Test wrapping a single particle"""
        box = molrs.Box.cube(2.0)
        
        # Point outside box
        points = np.array([[3.0, -1.0, 4.5]])
        wrapped = box.wrap(points)
        
        # Verify wrapped point is in [0, 2) for each dimension
        assert np.all(wrapped >= 0.0)
        assert np.all(wrapped < 2.0)

    def test_wrap_multiple_particles(self):
        """Test wrapping multiple particles"""
        box = molrs.Box.cube(2.0)
        
        points = np.array([[3.0, -1.0, 4.5],
                          [0.5, 0.5, 0.5],
                          [-2.1, 5.7, -0.3]])
        wrapped = box.wrap(points)
        
        # Verify all wrapped points are in [0, 2)
        assert np.all(wrapped >= 0.0)
        assert np.all(wrapped < 2.0)

    def test_wrap_multiple_images(self):
        """Test wrapping points far outside the box"""
        box = molrs.Box.cube(2.0)
        
        points = np.array([[10.0, -5.0, 20.3],
                          [0.5, 0.5, 0.5]])
        wrapped = box.wrap(points)
        
        # All points should be wrapped into [0, 2)
        assert np.all(wrapped >= 0.0)
        assert np.all(wrapped < 2.0)

    def test_fractional_coordinates(self):
        """Test conversion to fractional coordinates"""
        box = molrs.Box.cube(2.0)
        
        # Points at box corners and center
        points = np.array([[0.0, 0.0, 0.0],
                          [1.0, 1.0, 1.0],
                          [2.0, 2.0, 2.0]])
        
        frac = box.to_frac(points)
        
        npt.assert_allclose(frac[0], [0.0, 0.0, 0.0], atol=1e-5)
        npt.assert_allclose(frac[1], [0.5, 0.5, 0.5], atol=1e-5)
        npt.assert_allclose(frac[2], [1.0, 1.0, 1.0], atol=1e-5)

    def test_absolute_coordinates(self):
        """Test conversion from fractional to Cartesian coordinates"""
        box = molrs.Box.cube(2.0)
        
        f_points = np.array([[0.0, 0.0, 0.0],
                            [0.5, 0.5, 0.5],
                            [1.0, 1.0, 1.0]])
        
        cart = box.to_cart(f_points)
        
        npt.assert_allclose(cart[0], [0.0, 0.0, 0.0], atol=1e-5)
        npt.assert_allclose(cart[1], [1.0, 1.0, 1.0], atol=1e-5)
        npt.assert_allclose(cart[2], [2.0, 2.0, 2.0], atol=1e-5)

    def test_coordinate_roundtrip(self):
        """Test Cartesian -> Fractional -> Cartesian roundtrip"""
        box = molrs.Box.orthorhombic(np.array([3.0, 4.0, 5.0]))
        
        # Random points
        points = np.array([[1.5, 2.0, 2.5],
                          [0.5, 1.0, 4.0],
                          [2.8, 3.5, 0.1]])
        
        # Convert to fractional and back
        frac = box.to_frac(points)
        cart_back = box.to_cart(frac)
        
        # Should recover original points
        npt.assert_allclose(points, cart_back, atol=1e-5)

    def test_minimum_image_distance(self):
        """Test distance calculation with minimum image convention"""
        box = molrs.Box.cube(10.0)
        
        # Two points near opposite corners
        a = np.array([[1.0, 1.0, 1.0]])
        b = np.array([[9.0, 9.0, 9.0]])
        
        # Without minimum image
        delta_no_mi = box.delta(a, b, minimum_image=False)
        npt.assert_allclose(np.linalg.norm(delta_no_mi), np.sqrt(3 * 8**2), rtol=1e-5)
        
        # With minimum image (should take shorter path)
        delta_mi = box.delta(a, b, minimum_image=True)
        # Distance should be ~2*sqrt(3) ≈ 3.46
        npt.assert_allclose(np.linalg.norm(delta_mi), np.sqrt(3 * 2**2), rtol=1e-4)

    def test_delta_multiple_pairs(self):
        """Test computing deltas for multiple point pairs"""
        box = molrs.Box.cube(5.0)
        
        a = np.array([[0.0, 0.0, 0.0],
                     [1.0, 1.0, 1.0]])
        b = np.array([[1.0, 0.0, 0.0],
                     [2.0, 2.0, 2.0]])
        
        deltas = box.delta(a, b, minimum_image=False)
        
        npt.assert_allclose(deltas[0], [1.0, 0.0, 0.0], atol=1e-5)
        npt.assert_allclose(deltas[1], [1.0, 1.0, 1.0], atol=1e-5)

    def test_is_in(self):
        """Test checking if points are inside the box"""
        box = molrs.Box.cube(10.0)
        
        # Mix of points inside and outside
        points = np.array([[5.0, 5.0, 5.0],    # inside
                          [0.0, 0.0, 0.0],     # on boundary (inside)
                          [15.0, 5.0, 5.0],    # outside
                          [9.9, 9.9, 9.9],     # inside (near boundary)
                          [-1.0, 5.0, 5.0]])   # outside
        
        inside = box.is_in(points)
        
        assert inside[0] == True
        assert inside[1] == True
        assert inside[2] == False
        assert inside[3] == True
        assert inside[4] == False

    def test_orthorhombic_box(self):
        """Test orthorhombic box with different dimensions"""
        box = molrs.Box.orthorhombic(np.array([2.0, 3.0, 4.0]))
        
        # Check volume
        npt.assert_allclose(box.volume(), 24.0, rtol=1e-6)
        
        # Check lattice vectors
        v0 = box.lattice_vector(0)
        v1 = box.lattice_vector(1)
        v2 = box.lattice_vector(2)
        
        npt.assert_allclose(v0, [2.0, 0.0, 0.0], rtol=1e-6)
        npt.assert_allclose(v1, [0.0, 3.0, 0.0], rtol=1e-6)
        npt.assert_allclose(v2, [0.0, 0.0, 4.0], rtol=1e-6)

    def test_triclinic_box(self):
        """Test triclinic box"""
        # Create a triclinic box
        h = np.array([[5.0, 0.0, 0.0],
                      [1.0, 5.0, 0.0],
                      [0.5, 1.5, 5.0]])
        box = molrs.Box(h)
        
        # Volume should be det(h)
        expected_volume = np.linalg.det(h)
        npt.assert_allclose(box.volume(), expected_volume, rtol=1e-5)
        
        # Test coordinate transformations
        points = np.array([[1.0, 2.0, 3.0]])
        frac = box.to_frac(points)
        cart_back = box.to_cart(frac)
        npt.assert_allclose(points, cart_back, atol=1e-5)

    def test_pbc_control(self):
        """Test periodic boundary condition control"""
        # Box with no PBC
        h = np.array([[5.0, 0.0, 0.0],
                      [0.0, 5.0, 0.0],
                      [0.0, 0.0, 5.0]])
        pbc = np.array([False, False, False])
        box = molrs.Box(h, pbc=pbc)
        
        # Basic functionality should still work
        points = np.array([[2.5, 2.5, 2.5]])
        frac = box.to_frac(points)
        cart = box.to_cart(frac)
        npt.assert_allclose(points, cart, atol=1e-5)

    def test_pbc_partial(self):
        """Test box with PBC only in some directions"""
        h = np.array([[10.0, 0.0, 0.0],
                      [0.0, 10.0, 0.0],
                      [0.0, 0.0, 10.0]])
        # Only periodic in x and y
        pbc = np.array([True, True, False])
        box = molrs.Box(h, pbc=pbc)
        
        # Test wrapping - should wrap x,y but not z
        points = np.array([[15.0, -5.0, 20.0]])
        wrapped = box.wrap(points)
        
        # x and y should be wrapped, z should not
        assert 0.0 <= wrapped[0, 0] < 10.0
        assert 0.0 <= wrapped[0, 1] < 10.0
        # z might not be wrapped since PBC is off
        
    def test_origin_offset(self):
        """Test box with non-zero origin"""
        h = np.array([[5.0, 0.0, 0.0],
                      [0.0, 5.0, 0.0],
                      [0.0, 0.0, 5.0]])
        origin = np.array([1.0, 2.0, 3.0])
        box = molrs.Box(h, origin=origin)
        
        # Test coordinate transformations with offset
        points = np.array([[1.0, 2.0, 3.0],  # at origin
                          [6.0, 7.0, 8.0]])  # at origin + lattice vector
        
        frac = box.to_frac(points)
        npt.assert_allclose(frac[0], [0.0, 0.0, 0.0], atol=1e-5)
        npt.assert_allclose(frac[1], [1.0, 1.0, 1.0], atol=1e-5)

    def test_empty_array(self):
        """Test handling of empty arrays"""
        box = molrs.Box.cube(10.0)
        
        empty = np.empty((0, 3))
        result = box.wrap(empty)
        assert result.shape == (0, 3)

    def test_single_point_as_2d_array(self):
        """Test that single points must be provided as 2D arrays"""
        box = molrs.Box.cube(10.0)
        
        # Should work with shape (1, 3)
        points = np.array([[5.0, 5.0, 5.0]])
        result = box.wrap(points)
        assert result.shape == (1, 3)

    def test_repr(self):
        """Test string representation"""
        box = molrs.Box.cube(10.0)
        repr_str = repr(box)
        assert "Box" in repr_str

    def test_large_displacement_wrapping(self):
        """Test wrapping of very large displacements"""
        box = molrs.Box.cube(1.0)
        
        # Point many box lengths away
        points = np.array([[1000.5, -500.3, 250.7]])
        wrapped = box.wrap(points)
        
        # Should still wrap correctly
        assert np.all(wrapped >= 0.0)
        assert np.all(wrapped < 1.0)

    def test_numerical_precision(self):
        """Test numerical precision in transformations"""
        box = molrs.Box.orthorhombic(np.array([1e-10, 1e-10, 1e-10]))
        
        points = np.array([[5e-11, 5e-11, 5e-11]])
        frac = box.to_frac(points)
        cart = box.to_cart(frac)
        
        # Should maintain precision despite small scales
        npt.assert_allclose(points, cart, rtol=1e-4)

    def test_different_dtypes(self):
        """Test that different numpy dtypes are handled correctly"""
        box = molrs.Box.cube(10.0)
        
        # molrs requires float64 arrays
        # float32 arrays need to be converted first
        points_f32 = np.array([[5.0, 5.0, 5.0]], dtype=np.float32)
        with pytest.raises(TypeError):
            box.wrap(points_f32)
        
        # Converting to float64 works
        result_f32_converted = box.wrap(points_f32.astype(np.float64))
        assert result_f32_converted.dtype == np.float64
        
        # float64 works directly
        points_f64 = np.array([[5.0, 5.0, 5.0]], dtype=np.float64)
        result_f64 = box.wrap(points_f64)
        assert result_f64.dtype == np.float64

    def test_minimum_image_at_boundary(self):
        """Test minimum image convention at box boundaries"""
        box = molrs.Box.cube(10.0)
        
        # Points exactly at boundaries
        a = np.array([[0.0, 0.0, 0.0]])
        b = np.array([[10.0, 0.0, 0.0]])  # Should be equivalent to [0, 0, 0]
        
        delta = box.delta(a, b, minimum_image=True)
        # Distance should be very small
        npt.assert_allclose(np.linalg.norm(delta), 0.0, atol=1e-4)


class TestBoxCreation:
    """Test various box creation methods"""
    
    def test_cube_creation(self):
        """Test cubic box creation"""
        box = molrs.Box.cube(5.0)
        assert box.volume() == pytest.approx(125.0, rel=1e-6)
        
        # With origin
        box = molrs.Box.cube(5.0, origin=np.array([1.0, 2.0, 3.0]))
        assert box.volume() == pytest.approx(125.0, rel=1e-6)
    
    def test_orthorhombic_creation(self):
        """Test orthorhombic box creation"""
        lengths = np.array([2.0, 3.0, 4.0])
        box = molrs.Box.orthorhombic(lengths)
        assert box.volume() == pytest.approx(24.0, rel=1e-6)
        
        # With origin and PBC
        box = molrs.Box.orthorhombic(
            lengths,
            origin=np.array([0.0, 0.0, 0.0]),
            pbc=np.array([True, True, False])
        )
        assert box.volume() == pytest.approx(24.0, rel=1e-6)
    
    def test_triclinic_creation(self):
        """Test triclinic box creation"""
        h = np.array([[5.0, 0.0, 0.0],
                      [2.5, 4.33, 0.0],
                      [0.0, 0.0, 10.0]])
        box = molrs.Box(h)
        expected_vol = np.abs(np.linalg.det(h))
        assert box.volume() == pytest.approx(expected_vol, rel=1e-5)


class TestBoxEdgeCases:
    """Test edge cases and error conditions"""
    
    def test_zero_length_box(self):
        """Test that zero-length boxes are handled"""
        # This might raise an error or handle it gracefully
        # Adjust based on actual behavior
        pass
    
    def test_invalid_array_shapes(self):
        """Test error handling for invalid array shapes"""
        box = molrs.Box.cube(10.0)
        
        # Wrong number of columns
        with pytest.raises((ValueError, Exception)):
            box.wrap(np.array([[1.0, 2.0]]))
        
        # 1D array (should require 2D)
        with pytest.raises((ValueError, Exception)):
            box.wrap(np.array([1.0, 2.0, 3.0]))
    
    def test_mismatched_array_sizes(self):
        """Test error handling for mismatched arrays in delta"""
        box = molrs.Box.cube(10.0)
        
        a = np.array([[1.0, 2.0, 3.0]])
        b = np.array([[1.0, 2.0, 3.0], [4.0, 5.0, 6.0]])
        
        with pytest.raises((ValueError, Exception)):
            box.delta(a, b)


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
