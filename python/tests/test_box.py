import numpy as np
import pytest

import molrs


def test_cube_volume_and_vectors():
    b = molrs.Box.cube(2.0)
    assert pytest.approx(b.volume(), rel=1e-6) == 8.0
    a = b.lattice_vector(0)
    assert np.allclose(a, np.array([2.0, 0.0, 0.0], dtype=np.float32))


def test_ortho_to_frac_and_back():
    b = molrs.Box.ortho(np.array([2.0, 3.0, 4.0], dtype=np.float32))
    pts = np.array([[0.5, -1.0, 2.0], [2.5, 2.0, 6.0]], dtype=np.float32)
    frac = b.to_frac(pts)
    cart = b.to_cart(frac)
    assert np.allclose(pts, cart, atol=1e-5)


def test_wrap_and_isin():
    b = molrs.Box.cube(2.0)
    pts = np.array([[2.1, -0.1, 3.9], [-1.9, 4.2, 0.0]], dtype=np.float32)
    w = b.wrap(pts)
    f = b.to_frac(w)
    assert np.all((f >= 0.0) & (f < 1.0))
    inside = b.isin(w)
    assert inside.dtype == np.bool_
    assert inside.shape == (pts.shape[0],)
    assert inside.all()


def test_delta_minimum_image():
    b = molrs.Box.ortho(np.array([10.0, 10.0, 10.0], dtype=np.float32))
    p1 = np.array([[9.0, 9.0, 9.0]], dtype=np.float32)
    p2 = np.array([[1.0, 1.0, 1.0]], dtype=np.float32)
    d = b.delta(p1, p2, True)
    # distance should be 2 in each axis under MIC
    assert np.allclose(np.abs(d), np.array([[2.0, 2.0, 2.0]], dtype=np.float32), atol=1e-5)
